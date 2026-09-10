#!/usr/bin/env node
/**
 * Validate every cards JSON file against schema/cards.schema.json (draft 2020-12)
 * the official catalog oracle (cards/official/catalog.json), and the
 * printed-substring / reference rules in docs/schema.md § printed.
 * Live Cygames fetch is not part of this command.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

const require = createRequire(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CARDS = path.join(ROOT, "cards");
const SCHEMA = path.join(ROOT, "schema", "cards.schema.json");
const CATALOG_PATH = path.join(CARDS, "official", "catalog.json");
const CATALOG_FACTS = [
  "id",
  "name",
  "kind",
  "class",
  "tribes",
  "rarity",
  "cost",
  "attack",
  "defense",
  "set",
  "token",
  "text",
];

const errors = [];

function fail(file, msg) {
  errors.push(`${path.relative(ROOT, file)}: ${msg}`);
}

function walkJson(dir) {
  const out = [];
  if (!fs.existsSync(dir)) return out;
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      if (ent.name === "official") continue;
      out.push(...walkJson(p));
    } else if (ent.name.endsWith(".json")) out.push(p);
  }
  return out.sort();
}

function normWs(s) {
  return String(s ?? "")
    .replace(/\s+/g, " ")
    .trim();
}

function isSubstring(needle, haystack) {
  if (!needle) return false;
  return normWs(haystack).includes(normWs(needle));
}

function isSubstringAny(needle, haystacks) {
  return haystacks.some((h) => isSubstring(needle, h));
}

function factVal(obj, key) {
  if (key === "tribes") return obj.tribes ?? [];
  if (key === "attack" || key === "defense") return obj[key] ?? 0;
  return obj[key];
}

function factsEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

function modeHaystacks(data, rec) {
  const out = [data.text || ""];
  for (const se of rec?.specific_effects ?? []) {
    if (se.type === "accelerate" || se.type === "crystallize") {
      const label = se.type === "accelerate" ? "Accelerate" : "Crystallize";
      out.push(`${label} (${se.cost}): ${se.text}`);
      out.push(se.text);
    }
  }
  return out;
}

function catalogRecord(catalog, id) {
  if (id == null) return null;
  return catalog[String(id)] ?? null;
}

function numericCrestId(id) {
  const m = String(id ?? "").match(/^(?:crest|faith):(\d+)$/);
  return m ? m[1] : null;
}

function sentences(text) {
  const n = normWs(text);
  if (!n) return [];
  return n
    .split(/(?<=[.!?]["']?)\s+/)
    .map((s) => s.trim())
    .filter(Boolean);
}

function sentenceOverlap(printed, parentPrinted) {
  const p = normWs(printed);
  return sentences(parentPrinted).filter((s) => p.includes(s) || s.includes(p));
}

function walkEffects(node, visit) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const x of node) walkEffects(x, visit);
    return;
  }
  visit(node);
  if (Array.isArray(node.effects)) walkEffects(node.effects, visit);
  if (Array.isArray(node.then)) walkEffects(node.then, visit);
  if (Array.isArray(node.else)) walkEffects(node.else, visit);
  if (Array.isArray(node.options)) {
    for (const opt of node.options) {
      visit.option?.(opt, node);
      walkEffects(opt.effects, visit);
    }
  }
  if (Array.isArray(node.steps)) {
    for (const step of node.steps) {
      visit.step?.(step, node);
      walkEffects(step.effects, visit);
    }
  }
  if (node.ability && typeof node.ability === "object") {
    visit.nestedAbility?.(node.ability);
  }
}

function collectRefs(node, into) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const x of node) collectRefs(x, into);
    return;
  }
  if (typeof node.named === "string") into.cards.add(node.named);
  if (typeof node.gain === "string") into.crests.add(node.gain);
  if (typeof node.transformInto === "string") into.cards.add(node.transformInto);
  if (typeof node.card === "string" && /^\d{8}$/.test(node.card)) into.cards.add(node.card);
  if (typeof node.notCard === "string") into.cards.add(node.notCard);
  if (Array.isArray(node.cards)) {
    for (const id of node.cards) {
      if (typeof id === "string" && /^\d{8}$/.test(id)) into.cards.add(id);
    }
  }
  if (Array.isArray(node.requires)) {
    for (const id of node.requires) {
      if (typeof id === "string") into.cards.add(id);
    }
  }
  for (const v of Object.values(node)) collectRefs(v, into);
}

function stripPrinted(value) {
  if (Array.isArray(value)) return value.map(stripPrinted);
  if (value && typeof value === "object") {
    const out = {};
    for (const [k, v] of Object.entries(value)) {
      if (k === "printed") continue;
      out[k] = stripPrinted(v);
    }
    return out;
  }
  return value;
}

function canonical(value) {
  return JSON.stringify(value, (_, v) => {
    if (v && typeof v === "object" && !Array.isArray(v)) {
      return Object.fromEntries(Object.entries(v).sort(([a], [b]) => a.localeCompare(b)));
    }
    return v;
  });
}

function checkDuplicateEffects(file, data) {
  const abilityHashes = new Set();
  for (const ab of data.abilities ?? []) {
    for (const el of ab.effects ?? []) {
      abilityHashes.add(canonical(stripPrinted(el)));
    }
  }
  for (const mode of data.modes ?? []) {
    for (const el of mode.effects ?? []) {
      const hash = canonical(stripPrinted(el));
      if (abilityHashes.has(hash)) {
        fail(
          file,
          `mode copies an ability effect subtree (printed stripped): ${hash.slice(0, 80)}`,
        );
        return;
      }
    }
  }
  const optionLists = [];
  const visit = (node) => {
    if (!node || typeof node !== "object") return;
    if (Array.isArray(node)) {
      for (const x of node) visit(x);
      return;
    }
    if (node.op === "choose" && Array.isArray(node.options)) {
      optionLists.push(canonical(stripPrinted(node.options)));
    }
    for (const v of Object.values(node)) visit(v);
  };
  visit(data);
  const seenOpts = new Set();
  for (const h of optionLists) {
    if (seenOpts.has(h)) {
      fail(file, "identical choose options list appears twice in this file");
      return;
    }
    seenOpts.add(h);
  }
}

function checkFaithGain(file, node) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const x of node) checkFaithGain(file, x);
    return;
  }
  if (typeof node.gain === "string" && node.gain.startsWith("faith:")) {
    fail(file, `CardSource/crest gain must not reference a faith: id (${node.gain})`);
  }
  if (typeof node.named === "string" && node.named.startsWith("faith:")) {
    fail(file, `CardSource must not reference a faith: id (${node.named})`);
  }
  for (const v of Object.values(node)) checkFaithGain(file, v);
}

function abilityHasChoose(ability) {
  if (!ability || !Array.isArray(ability.effects)) return false;
  return ability.effects.some((e) => e && e.op === "choose");
}

function checkOptionsFrom(file, data) {
  const byOn = new Map();
  for (const ab of data.abilities ?? []) {
    if (ab?.on) byOn.set(ab.on, ab);
  }
  const visit = (node) => {
    if (!node || typeof node !== "object") return;
    if (Array.isArray(node)) {
      for (const x of node) visit(x);
      return;
    }
    if (node.op === "choose" && node.optionsFrom) {
      const src = byOn.get(node.optionsFrom);
      if (!src) {
        fail(file, `optionsFrom ${node.optionsFrom} has no ability with that trigger on this card`);
      } else if (!abilityHasChoose(src)) {
        fail(
          file,
          `optionsFrom ${node.optionsFrom} does not point at a clause root that is a choose`,
        );
      }
    }
    for (const v of Object.values(node)) visit(v);
  };
  visit(data);
}

function checkPrintedTree(file, parentPrinted, effects, label, cardText) {
  if (!Array.isArray(effects)) return;
  const used = new Set();
  for (const [i, root] of effects.entries()) {
    if (!root || typeof root !== "object") continue;
    if (!root.printed) {
      fail(file, `${label}[${i}] missing printed`);
      continue;
    }
    if (!isSubstring(root.printed, parentPrinted)) {
      fail(
        file,
        `${label}[${i}] printed is not a whitespace-normalised substring of its parent`,
      );
    }
    const overlap = sentenceOverlap(root.printed, parentPrinted);
    for (const s of overlap) {
      if (used.has(s)) {
        fail(
          file,
          `${label}[${i}] shares a sentence with another clause root: "${s}"`,
        );
      }
      used.add(s);
    }
    walkEffects(
      root,
      Object.assign(
        (node) => {
          if (node.op === "choose" && Array.isArray(node.options)) {
            for (const [j, opt] of node.options.entries()) {
              if (!opt.printed) {
                fail(file, `${label}[${i}] choose option ${j} missing printed`);
              } else if (
                !isSubstring(opt.printed, parentPrinted) &&
                !isSubstring(opt.printed, cardText || "")
              ) {
                fail(
                  file,
                  `${label}[${i}] choose option ${j} printed is not a substring of its parent or card text`,
                );
              }
            }
          }
          if (node.op === "sequence" && Array.isArray(node.steps)) {
            for (const [j, step] of node.steps.entries()) {
              if (!step.printed) {
                fail(file, `${label}[${i}] sequence step ${j} missing printed`);
              } else if (
                !isSubstring(step.printed, parentPrinted) &&
                !isSubstring(step.printed, cardText || "")
              ) {
                fail(
                  file,
                  `${label}[${i}] sequence step ${j} printed is not a substring of its parent or card text`,
                );
              }
            }
          }
          if (node.op === "randomSplit" && Array.isArray(node.effects) && node.printed) {
            checkPrintedTree(
              file,
              node.printed,
              node.effects,
              `${label}[${i}].randomSplit`,
              cardText,
            );
          }
        },
        {
          nestedAbility: (ability) => {
            checkAbility(file, ability, parentPrinted, `${label}[${i}].grantedAbility`);
          },
        },
      ),
    );
  }
}

function checkAbility(file, ability, textHaystack, label) {
  if (!ability.printed) {
    fail(file, `${label} missing printed`);
    return;
  }
  if (!isSubstring(ability.printed, textHaystack)) {
    fail(
      file,
      `${label} printed is not a whitespace-normalised substring of card/crest/mode text`,
    );
  }
  if (ability.on === "static") return;
  checkPrintedTree(file, ability.printed, ability.effects, `${label}.effects`, textHaystack);
}

function grantedAbilityHasCrestGain(data, crestId) {
  const found = { yes: false };
  const visit = (node) => {
    if (!node || typeof node !== "object") return;
    if (Array.isArray(node)) {
      for (const x of node) visit(x);
      return;
    }
    if (node.op === "crest" && node.gain === crestId) found.yes = true;
    for (const v of Object.values(node)) visit(v);
  };
  visit(data);
  return found.yes;
}

function checkAgainstCatalog(file, data, catalog) {
  const isCrest = typeof data.faith === "boolean" && Array.isArray(data.grantedBy);
  if (isCrest) {
    const typ = data.faith ? "faith" : "crest";
    const ids = [numericCrestId(data.id), ...(data.grantedBy ?? []).map(String)].filter(Boolean);
    let official = null;
    for (const id of ids) {
      const rec = catalogRecord(catalog, id);
      const se = rec?.specific_effects?.find((e) => e.type === typ);
      if (se) {
        official = se.text;
        break;
      }
    }
    if (official == null) {
      fail(file, `no official ${typ} specific-effect text for ${data.id}`);
      return;
    }
    if (data.text !== official) {
      fail(file, `${typ} text does not equal official catalog specific-effect text`);
    }
    return;
  }
  if (data.id == null) return;
  const rec = catalogRecord(catalog, data.id);
  if (!rec) {
    fail(file, `id ${data.id} is not in the official catalog`);
    return;
  }
  for (const key of CATALOG_FACTS) {
    if (!factsEqual(factVal(data, key), factVal(rec, key))) {
      fail(file, `${key} does not equal official catalog record`);
    }
  }
}

function schemaOnly(dirArg) {
  const schema = JSON.parse(fs.readFileSync(SCHEMA, "utf8"));
  const ajv = new Ajv2020({
    strict: true,
    allErrors: true,
    allowUnionTypes: true,
  });
  addFormats(ajv);
  const validate = ajv.compile(schema);

  const dir = path.isAbsolute(dirArg)
    ? dirArg
    : fs.existsSync(path.resolve(dirArg))
      ? path.resolve(dirArg)
      : path.join(ROOT, dirArg);
  const files = walkJson(dir);
  if (files.length === 0) {
    fail(dir, "no card files found");
  }
  for (const file of files) {
    let data;
    try {
      data = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch (e) {
      fail(file, `invalid JSON: ${e.message}`);
      continue;
    }
    if (!validate(data)) {
      for (const err of validate.errors ?? []) {
        fail(file, `schema ${err.instancePath || "/"} ${err.message}`);
      }
    }
  }
  if (errors.length) {
    for (const e of errors) console.error(e);
    console.error(`\n${errors.length} error(s), ${files.length} file(s)`);
    process.exit(1);
  }
  console.log(`ok: ${files.length} files schema-only`);
}

function main() {
  const argv = process.argv.slice(2);
  if (argv[0] === "--schema-only") {
    if (!argv[1]) {
      console.error("usage: node validate.mjs --schema-only <dir>");
      process.exit(1);
    }
    schemaOnly(argv[1]);
    return;
  }

  const schema = JSON.parse(fs.readFileSync(SCHEMA, "utf8"));
  const ajv = new Ajv2020({
    strict: true,
    allErrors: true,
    allowUnionTypes: true,
  });
  addFormats(ajv);
  const validate = ajv.compile(schema);

  if (!fs.existsSync(CATALOG_PATH)) {
    fail(CATALOG_PATH, "missing official catalog (cards/official/catalog.json)");
  }
  const catalog = fs.existsSync(CATALOG_PATH)
    ? JSON.parse(fs.readFileSync(CATALOG_PATH, "utf8"))
    : {};

  const files = walkJson(CARDS);
  if (files.length === 0) {
    fail(CARDS, "no card files found");
  }

  const byId = new Map();
  const docs = [];

  for (const file of files) {
    const base = path.basename(file);
    if (base.includes(":")) {
      fail(file, "file name contains a colon (use crest-… / faith-… on NTFS)");
    }
    let data;
    try {
      data = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch (e) {
      fail(file, `invalid JSON: ${e.message}`);
      continue;
    }
    docs.push({ file, data });
    if (!validate(data)) {
      for (const err of validate.errors ?? []) {
        fail(file, `schema ${err.instancePath || "/"} ${err.message}`);
      }
    }
    if (data.id) {
      if (byId.has(data.id)) {
        fail(file, `duplicate id ${data.id} (also ${path.relative(ROOT, byId.get(data.id))})`);
      }
      byId.set(data.id, file);
    }
    const isCrest = typeof data.faith === "boolean" && Array.isArray(data.grantedBy);
    if (!isCrest) {
      const emptyOk =
        !data.text &&
        !(data.abilities && data.abilities.length) &&
        !(data.modes && data.modes.length) &&
        !data.fuse;
      if (!data.text && !emptyOk) {
        fail(file, "text is empty");
      }
    } else if (!data.text) {
      fail(file, "crest text is empty");
    }

    checkFaithGain(file, data);
    checkDuplicateEffects(file, data);
    checkOptionsFrom(file, data);
    checkAgainstCatalog(file, data, catalog);

    if (isCrest) {
      for (const [i, ab] of (data.abilities ?? []).entries()) {
        checkAbility(file, ab, data.text, `abilities[${i}]`);
      }
    } else {
      const rec = catalogRecord(catalog, data.id);
      const hay = modeHaystacks(data, rec);
      for (const [i, ab] of (data.abilities ?? []).entries()) {
        checkAbility(file, ab, data.text, `abilities[${i}]`);
      }
      for (const [i, mode] of (data.modes ?? []).entries()) {
        if (!mode.printed) {
          fail(file, `modes[${i}] missing printed`);
          continue;
        }
        if (!isSubstringAny(mode.printed, hay)) {
          fail(
            file,
            `modes[${i}] printed is not a substring of text or official Accelerate/Crystallize line`,
          );
        }
        const modeHay = [mode.printed, ...hay].join("\n");
        checkPrintedTree(file, mode.printed, mode.effects ?? [], `modes[${i}].effects`, modeHay);
        for (const [j, ab] of (mode.abilities ?? []).entries()) {
          checkAbility(file, ab, mode.printed, `modes[${i}].abilities[${j}]`);
        }
      }
    }
  }

  for (const { file, data } of docs) {
    const refs = { cards: new Set(), crests: new Set() };
    collectRefs(data, refs);
    for (const id of refs.cards) {
      if (!byId.has(id)) fail(file, `dangling card id ${id}`);
    }
    for (const id of refs.crests) {
      if (!byId.has(id)) fail(file, `dangling crest id ${id}`);
    }
  }

  for (const { file, data } of docs) {
    const isCrest = typeof data.faith === "boolean" && Array.isArray(data.grantedBy);
    if (!isCrest || data.faith) continue;
    for (const gid of data.grantedBy ?? []) {
      const gfile = byId.get(gid);
      if (!gfile) {
        fail(file, `grantedBy ${gid} is not an authored card file`);
        continue;
      }
      const granter = docs.find((d) => d.file === gfile)?.data;
      if (!granter || !grantedAbilityHasCrestGain(granter, data.id)) {
        fail(
          file,
          `grantedBy ${gid} must contain a crest {gain} of ${data.id} (faith files exempt)`,
        );
      }
    }
  }

  if (errors.length) {
    for (const e of errors) console.error(e);
    console.error(`\n${errors.length} error(s), ${files.length} file(s)`);
    process.exit(1);
  }
  console.log(`ok: ${files.length} files validated`);
}

main();
