#!/usr/bin/env node
/**
 * Fetch Cygames' keyword glossary into cards/official/glossary.{en,ja}.json
 * and rules/official-glossary.md.
 *   node tools/fetch-glossary.mjs           write JSON + markdown
 *   node tools/fetch-glossary.mjs --check     re-fetch; exit 1 if entries differ
 *   node tools/fetch-glossary.mjs --md-from-json  rewrite markdown from committed JSON
 *
 * No dependencies. ≤ 4 req/s. One retry on 5xx.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = "https://shadowverse-wb.com/web/System/glossaryList";
const EN_OUT = path.join(ROOT, "cards", "official", "glossary.en.json");
const JA_OUT = path.join(ROOT, "cards", "official", "glossary.ja.json");
const MD_OUT = path.join(ROOT, "rules", "official-glossary.md");
const MIN_GAP_MS = 250;

/** EN/JA pairs where the two languages differ in substance (not just wording). */
const SUBSTANCE_NOTES = {
  Fuse:
    'EN "You can only fuse once per turn" vs JA 「融合を持つ手札のカードは、1ターンに1回…融合できます」 — once per turn **per card** (each Fuse holder in hand gets its own once-per-turn fuse).',
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
  t = t.replace(/<br\s*\/?>/gi, "\n");
  t = t.replace(/<[^>]+>/g, "");
  return t;
}

function textToMarkdown(text) {
  return stripMarkup(text)
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

async function fetchList(lang) {
  const headers = { Lang: lang, Accept: "application/json" };
  const once = async () => {
    const res = await fetch(SOURCE, { headers });
    return res;
  };
  let res = await once();
  if (res.status >= 500 && res.status <= 599) {
    await sleep(MIN_GAP_MS);
    res = await once();
  }
  if (!res.ok) {
    throw new Error(`GET ${SOURCE} (Lang: ${lang}) → HTTP ${res.status} ${res.statusText}`);
  }
  const body = await res.json();
  const list = body?.data?.glossary_list;
  if (!Array.isArray(list) || list.length === 0) {
    throw new Error(`empty glossary_list for Lang: ${lang}`);
  }
  return list.map((row, ordinal) => ({
    ordinal,
    title: String(row.title ?? ""),
    text: String(row.text ?? ""),
  }));
}

export async function fetchGlossaryPair() {
  let last = 0;
  const wait = MIN_GAP_MS - (Date.now() - last);
  if (last && wait > 0) await sleep(wait);
  last = Date.now();
  const en = await fetchList("en");
  const wait2 = MIN_GAP_MS - (Date.now() - last);
  if (wait2 > 0) await sleep(wait2);
  last = Date.now();
  const ja = await fetchList("ja");
  if (en.length !== ja.length) {
    throw new Error(`glossary length mismatch: en=${en.length} ja=${ja.length}`);
  }
  return { en, ja };
}

function buildCatalog(lang, entries, fetchedAt) {
  const collator = lang === "ja" ? "ja" : "en";
  const sorted = [...entries].sort((a, b) => a.title.localeCompare(b.title, collator));
  return {
    _meta: {
      source: SOURCE,
      fetched_at: fetchedAt,
      lang,
      count: sorted.length,
    },
    entries: sorted,
  };
}

export function officialGlossaryMarkdown(enCatalog, jaCatalog) {
  const fetched = (enCatalog._meta?.fetched_at || "").slice(0, 10);
  const count = enCatalog._meta?.count ?? enCatalog.entries?.length ?? 0;
  const jaByOrdinal = new Map((jaCatalog.entries || []).map((e) => [e.ordinal, e]));
  const enOrder = [...(enCatalog.entries || [])].sort((a, b) =>
    a.title.localeCompare(b.title, "en"),
  );

  let md = "# Official Cygames keyword glossary\n\n";
  md += `Fetched ${fetched} from ${SOURCE}. ${count} entries (English headings; Japanese is the original client text).\n\n`;

  for (const en of enOrder) {
    const ja = jaByOrdinal.get(en.ordinal);
    md += `## ${en.title}\n\n`;
    md += `${textToMarkdown(en.text)}\n\n`;
    if (ja) {
      md += `> **${ja.title}** — ${textToMarkdown(ja.text).replace(/\n/g, "\n> ")}\n\n`;
    }
    const note = SUBSTANCE_NOTES[en.title];
    if (note) {
      md += `_EN/JA substance note:_ ${note}\n\n`;
    }
  }
  return md;
}

function entriesOnly(catalog) {
  return catalog.entries ?? [];
}

function writeOutputs(enCatalog, jaCatalog) {
  fs.mkdirSync(path.dirname(EN_OUT), { recursive: true });
  fs.writeFileSync(EN_OUT, JSON.stringify(enCatalog, null, 2) + "\n");
  fs.writeFileSync(JA_OUT, JSON.stringify(jaCatalog, null, 2) + "\n");
  fs.writeFileSync(MD_OUT, officialGlossaryMarkdown(enCatalog, jaCatalog));
  console.log(
    `wrote ${EN_OUT}, ${JA_OUT}, ${MD_OUT} (${enCatalog._meta.count} entries, fetched ${enCatalog._meta.fetched_at})`,
  );
}

async function main() {
  if (process.argv.includes("--md-from-json")) {
    if (!fs.existsSync(EN_OUT) || !fs.existsSync(JA_OUT)) {
      console.error("missing committed glossary JSON");
      process.exit(1);
    }
    const en = JSON.parse(fs.readFileSync(EN_OUT, "utf8"));
    const ja = JSON.parse(fs.readFileSync(JA_OUT, "utf8"));
    fs.writeFileSync(MD_OUT, officialGlossaryMarkdown(en, ja));
    console.log(`wrote ${MD_OUT}`);
    return;
  }

  const check = process.argv.includes("--check");
  const { en, ja } = await fetchGlossaryPair();
  const fetchedAt = new Date().toISOString();
  const enCatalog = buildCatalog("en", en, fetchedAt);
  const jaCatalog = buildCatalog("ja", ja, fetchedAt);

  if (check) {
    if (!fs.existsSync(EN_OUT) || !fs.existsSync(JA_OUT)) {
      console.error("missing committed glossary JSON");
      process.exit(1);
    }
    const committedEn = JSON.parse(fs.readFileSync(EN_OUT, "utf8"));
    const committedJa = JSON.parse(fs.readFileSync(JA_OUT, "utf8"));
    const a = JSON.stringify(entriesOnly(enCatalog));
    const b = JSON.stringify(entriesOnly(committedEn));
    const c = JSON.stringify(entriesOnly(jaCatalog));
    const d = JSON.stringify(entriesOnly(committedJa));
    if (a !== b || c !== d) {
      console.error("official glossary differs from committed cards/official/glossary.*.json");
      process.exit(1);
    }
    console.log("ok: live official glossary matches committed entries (fetched_at ignored)");
    return;
  }

  writeOutputs(enCatalog, jaCatalog);
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
