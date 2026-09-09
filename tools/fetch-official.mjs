#!/usr/bin/env node
/**
 * Fetch Cygames' card list into cards/official/catalog.json.
 *   node tools/fetch-official.mjs                 write catalog + official-qa.md
 *   node tools/fetch-official.mjs --check         re-fetch; exit 1 if records differ
 *   node tools/fetch-official.mjs --qa-from-catalog  rewrite official-qa.md from committed catalog
 *
 * No dependencies. ≤ 4 req/s. One retry on 5xx.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT = path.join(ROOT, "cards", "official", "catalog.json");
const SOURCE = "https://shadowverse-wb.com/web/CardList/cardList";
const LANG = "en";
const MIN_GAP_MS = 250;

const TYPE = {
  1: "follower",
  2: "amulet",
  3: "amulet",
  4: "spell",
};
const CLASS = {
  0: "neutral",
  1: "forestcraft",
  2: "swordcraft",
  3: "runecraft",
  4: "dragoncraft",
  5: "abysscraft",
  6: "havencraft",
  7: "portalcraft",
};
const RARITY = {
  1: "bronze",
  2: "silver",
  3: "gold",
  4: "legendary",
};
const TRIBE = {
  2: "officer",
  3: "luminous",
  4: "levin",
  5: "pixie",
  6: "departed",
  8: "earth sigil",
  11: "mysteria",
  12: "golem",
  13: "shikigami",
  14: "artifact",
  15: "puppetry",
  17: "marine",
  18: "loot",
  19: "encroacher",
  20: "anathema",
};
const SE_TYPE = {
  1: "crest",
  2: "crystallize",
  3: "accelerate",
  4: "faith",
};

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function stripMarkup(s) {
  if (s == null) return "";
  let t = String(s);
  t = t.replace(/<hr\s*\/?>/gi, "\n");
  t = t.replace(/<\/?(?:b|i)>/gi, "");
  t = t.replace(/<\/?color(?:=[^>]*)?>/gi, "");
  t = t.replace(/<\/?ridx(?:=[^>]*)?>/gi, "");
  t = t.replace(/<[^>]+>/g, "");
  return t;
}

function padId(n) {
  return String(n);
}

async function getPage(offset) {
  const url = `${SOURCE}?offset=${offset}&include_token=1`;
  const headers = { Lang: LANG, Accept: "application/json" };
  const once = async () => {
    const res = await fetch(url, { headers });
    return res;
  };
  let res = await once();
  if (res.status >= 500 && res.status <= 599) {
    await sleep(MIN_GAP_MS);
    res = await once();
  }
  if (!res.ok) {
    throw new Error(`GET ${url} → HTTP ${res.status} ${res.statusText}`);
  }
  return res.json();
}

function isStarterSet(setId) {
  const n = Number(setId);
  return n >= 80000 && n < 90000;
}

function hashOrNull(v) {
  if (v == null || v === "") return null;
  return String(v);
}

function evoHashes(evo) {
  if (!evo || typeof evo !== "object" || Array.isArray(evo)) {
    return { evo_card_image_hash: null, evo_card_banner_image_hash: null };
  }
  return {
    evo_card_image_hash: hashOrNull(evo.card_image_hash),
    evo_card_banner_image_hash: hashOrNull(evo.card_banner_image_hash),
  };
}

function normalizeCommon(common, extras) {
  const set = Number(common.card_set_id);
  const type = Number(common.type);
  const klass = Number(common.class);
  const rarity = Number(common.rarity);
  const tribes = (common.tribes || [])
    .map(Number)
    .filter((t) => t !== 0)
    .map((t) => {
      if (!TRIBE[t]) throw new Error(`unknown tribe ${t} on ${common.card_id}`);
      return TRIBE[t];
    });
  if (!TYPE[type]) throw new Error(`unknown type ${type} on ${common.card_id}`);
  if (!CLASS[klass]) throw new Error(`unknown class ${klass} on ${common.card_id}`);
  if (!RARITY[rarity]) throw new Error(`unknown rarity ${rarity} on ${common.card_id}`);
  const raw = common.skill_text ?? "";
  return {
    id: padId(common.card_id),
    name: common.name,
    kind: TYPE[type],
    class: CLASS[klass],
    tribes,
    rarity: RARITY[rarity],
    cost: Number(common.cost) || 0,
    attack: Number(common.atk) || 0,
    defense: Number(common.life) || 0,
    set,
    token: Boolean(common.is_token),
    rotation: Boolean(common.is_include_rotation),
    deck_enabled_num: Number(common.deck_enabled_num) || 0,
    related_card_ids: extras.related,
    specific_effect_card_ids: extras.seIds,
    card_image_hash: hashOrNull(common.card_image_hash),
    card_banner_image_hash: hashOrNull(common.card_banner_image_hash),
    evo_card_image_hash: extras.evo_card_image_hash,
    evo_card_banner_image_hash: extras.evo_card_banner_image_hash,
    text: stripMarkup(raw),
    text_raw: raw,
    questions: Array.isArray(common.questions) ? common.questions : [],
    specific_effects: extras.effects,
  };
}

function effectsFor(seIds, seInfo) {
  const out = [];
  for (const eid of seIds) {
    const info = seInfo[String(eid)];
    if (!info) throw new Error(`dangling specific_effect_card_id ${eid}`);
    const t = SE_TYPE[Number(info.specific_effect_type)];
    if (!t) {
      throw new Error(
        `unknown specific_effect_type ${info.specific_effect_type} on ${eid}`,
      );
    }
    const raw = info.skill_text ?? "";
    out.push({
      type: t,
      cost: Number(info.cost) || 0,
      text: stripMarkup(raw),
      text_raw: raw,
    });
  }
  return out;
}

export async function fetchCatalog() {
  const details = new Map();
  const cardsMeta = new Map();
  const seInfo = {};
  const sortIds = [];
  let count = null;
  let offset = 0;
  let last = 0;

  while (true) {
    const wait = MIN_GAP_MS - (Date.now() - last);
    if (last && wait > 0) await sleep(wait);
    last = Date.now();
    const body = await getPage(offset);
    const data = body.data;
    if (count == null) count = Number(data.count);
    const page = data.sort_card_id_list || [];
    if (page.length === 0 && offset < count) {
      throw new Error(`empty page at offset ${offset} (count ${count})`);
    }
    for (const id of page) sortIds.push(String(id));
    for (const [id, det] of Object.entries(data.card_details || {})) {
      if (!details.has(id)) details.set(id, det);
    }
    for (const [id, meta] of Object.entries(data.cards || {})) {
      if (!cardsMeta.has(id)) cardsMeta.set(id, meta);
    }
    Object.assign(seInfo, data.specific_effect_card_info || {});
    offset += page.length;
    if (offset >= count) break;
  }

  const distinct = new Set(sortIds);
  if (distinct.size !== count) {
    throw new Error(
      `sort_card_id_list union has ${distinct.size} distinct ids; API count is ${count}`,
    );
  }

  const evoMismatches = [];
  const records = {};
  for (const id of [...distinct].sort()) {
    const det = details.get(id);
    if (!det || !det.common) {
      throw new Error(`sort id ${id} missing card_details.common`);
    }
    if (isStarterSet(det.common.card_set_id)) continue;
    const evo = det.evo;
    if (evo && typeof evo === "object" && !Array.isArray(evo) && "skill_text" in evo) {
      if ((evo.skill_text ?? "") !== (det.common.skill_text ?? "")) {
        evoMismatches.push({
          id,
          name: det.common.name,
          common: det.common.skill_text,
          evo: evo.skill_text,
        });
      }
    }
    const meta = cardsMeta.get(id) || {};
    const related = (meta.related_card_ids || []).map(String);
    const seIds = (meta.specific_effect_card_ids || []).map(String);
    records[id] = normalizeCommon(det.common, {
      related,
      seIds,
      effects: effectsFor(seIds, seInfo),
      ...evoHashes(evo),
    });
  }

  const catalog = {
    _meta: {
      source: SOURCE,
      fetched_at: new Date().toISOString(),
      lang: LANG,
      count,
      maps: {
        type: TYPE,
        class: CLASS,
        rarity: RARITY,
        tribe: TRIBE,
        specific_effect_type: SE_TYPE,
      },
      evo_skill_text_mismatches: evoMismatches,
    },
    ...records,
  };
  return catalog;
}

function recordsOnly(catalog) {
  const out = {};
  for (const [k, v] of Object.entries(catalog)) {
    if (k === "_meta") continue;
    out[k] = v;
  }
  return out;
}

export function officialQaMarkdown(catalog) {
  const fetched = (catalog._meta?.fetched_at || "").slice(0, 10);
  const lang = catalog._meta?.lang || LANG;
  const entries = [];
  for (const [id, rec] of Object.entries(catalog)) {
    if (id === "_meta") continue;
    const qs = rec.questions || [];
    if (!qs.length) continue;
    entries.push({ id, name: rec.name, qs });
  }
  entries.sort((a, b) => a.id.localeCompare(b.id));
  const nQ = entries.reduce((n, e) => n + e.qs.length, 0);
  const count = catalog._meta?.count ?? entries.length;
  let md = "# Official Cygames per-card Q&A\n\n";
  md += `Fetched ${fetched} from https://shadowverse-wb.com (lang=${lang}). ${nQ} Q&A entries across ${entries.length} cards (${count} catalog ids).\n`;
  for (const e of entries) {
    md += `\n## ${e.id} ${e.name}\n`;
    for (const qa of e.qs) {
      const q = String(qa.question ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
      const a = String(qa.answer ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
      md += `\n**Q:** ${q}\n\n**A:** ${a}\n`;
    }
  }
  return md;
}

export function derivePool(catalog) {
  const byId = recordsOnly(catalog);
  const rotation = [];
  for (const [id, rec] of Object.entries(byId)) {
    if (rec.rotation && !rec.token) rotation.push(id);
  }
  const seen = new Set(rotation);
  const queue = [...rotation];
  const tokens = [];
  const missingRelated = [];
  while (queue.length) {
    const id = queue.shift();
    const rec = byId[id];
    if (!rec) continue;
    for (const rid of rec.related_card_ids || []) {
      const r = byId[rid];
      if (!r) {
        missingRelated.push({ from: id, related: rid });
        continue;
      }
      if (seen.has(rid)) continue;
      if (!r.token) continue;
      seen.add(rid);
      tokens.push(rid);
      queue.push(rid);
    }
  }
  const pool = [...rotation, ...tokens];
  const se = { crest: 0, faith: 0, crystallize: 0, accelerate: 0 };
  for (const id of pool) {
    for (const e of byId[id].specific_effects || []) {
      if (se[e.type] != null) se[e.type] += 1;
    }
  }
  return {
    rotation: rotation.length,
    tokens: tokens.length,
    specific_effects: se,
    missingRelated,
  };
}

function writeOfficialQa(catalog) {
  const qaPath = path.join(ROOT, "rules", "official-qa.md");
  fs.writeFileSync(qaPath, officialQaMarkdown(catalog));
  console.log(`wrote ${qaPath}`);
}

function mainWrite(catalog) {
  fs.mkdirSync(path.dirname(OUT), { recursive: true });
  fs.writeFileSync(OUT, JSON.stringify(catalog, null, 2) + "\n");
  writeOfficialQa(catalog);
  const n = Object.keys(catalog).length - 1;
  const pool = derivePool(catalog);
  console.log(
    `wrote ${OUT} (${n} cards, API count ${catalog._meta.count}, evo mismatches ${catalog._meta.evo_skill_text_mismatches.length})`,
  );
  console.log(
    `pool from catalog: ${pool.rotation} rotation / ${pool.tokens} reachable tokens / ${pool.specific_effects.crest} crest + ${pool.specific_effects.faith} faith (+ ${pool.specific_effects.crystallize} crystallize + ${pool.specific_effects.accelerate} accelerate)`,
  );
}

async function main() {
  if (process.argv.includes("--qa-from-catalog")) {
    if (!fs.existsSync(OUT)) {
      console.error(`missing committed catalog ${OUT}`);
      process.exit(1);
    }
    writeOfficialQa(JSON.parse(fs.readFileSync(OUT, "utf8")));
    return;
  }
  const check = process.argv.includes("--check");
  const catalog = await fetchCatalog();
  if (check) {
    if (!fs.existsSync(OUT)) {
      console.error(`missing committed catalog ${OUT}`);
      process.exit(1);
    }
    const committed = JSON.parse(fs.readFileSync(OUT, "utf8"));
    const a = JSON.stringify(recordsOnly(catalog));
    const b = JSON.stringify(recordsOnly(committed));
    if (a !== b) {
      console.error("official catalog differs from committed cards/official/catalog.json");
      process.exit(1);
    }
    console.log("ok: live official catalog matches committed records (fetched_at ignored)");
    return;
  }
  mainWrite(catalog);
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
