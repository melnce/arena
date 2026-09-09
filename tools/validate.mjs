#!/usr/bin/env node
/**
 * Validate every cards JSON file against schema/cards.schema.json (draft 2020-12)
 * and the printed-substring / reference rules in docs/schema.md § printed.
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

const errors = [];

function fail(file, msg) {
  errors.push(`${path.relative(ROOT, file)}: ${msg}`);
}

function walkJson(dir) {
  const out = [];
  if (!fs.existsSync(dir)) return out;
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) out.push(...walkJson(p));
    else if (ent.name.endsWith(".json")) out.push(p);
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
    for (const step of node.steps) walkEffects(step.effects, visit);
  }
  if (node.ability && typeof node.ability === "object") {
    visit.nestedAbility?.(node.ability);
  }
}

function collectNamedRefs(node, into) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const x of node) collectNamedRefs(x, into);
    return;
  }
  if (typeof node.named === "string") into.cards.add(node.named);
  if (typeof node.gain === "string") into.crests.add(node.gain);
  if (typeof node.transformInto === "string") into.cards.add(node.transformInto);
  for (const v of Object.values(node)) collectNamedRefs(v, into);
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
    walkEffects(root, Object.assign((node) => {
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
      if (node.op === "randomSplit" && Array.isArray(node.effects) && node.printed) {
        checkPrintedTree(file, node.printed, node.effects, `${label}[${i}].randomSplit`, cardText);
      }
    }, {
      nestedAbility: (ability) => {
        checkAbility(file, ability, parentPrinted, `${label}[${i}].grantedAbility`);
      },
    }));
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
  checkPrintedTree(file, ability.printed, ability.effects, `${label}.effects`, textHaystack);
}

function main() {
  const schema = JSON.parse(fs.readFileSync(SCHEMA, "utf8"));
  const ajv = new Ajv2020({
    strict: true,
    allErrors: true,
    allowUnionTypes: true,
  });
  addFormats(ajv);
  const validate = ajv.compile(schema);

  const files = walkJson(CARDS);
  if (files.length === 0) {
    fail(CARDS, "no card files found");
  }

  const byId = new Map();
  const docs = [];

  for (const file of files) {
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

    if (isCrest) {
      for (const [i, ab] of (data.abilities ?? []).entries()) {
        checkAbility(file, ab, data.text, `abilities[${i}]`);
      }
    } else {
      for (const [i, ab] of (data.abilities ?? []).entries()) {
        checkAbility(file, ab, data.text, `abilities[${i}]`);
      }
      for (const [i, mode] of (data.modes ?? []).entries()) {
        if (!mode.printed) {
          fail(file, `modes[${i}] missing printed`);
          continue;
        }
        if (!isSubstring(mode.printed, data.text)) {
          fail(file, `modes[${i}] printed is not a substring of text`);
        }
        checkPrintedTree(file, mode.printed, mode.effects ?? [], `modes[${i}].effects`, data.text);
        for (const [j, ab] of (mode.abilities ?? []).entries()) {
          checkAbility(file, ab, mode.printed, `modes[${i}].abilities[${j}]`);
        }
      }
    }
  }

  for (const { file, data } of docs) {
    const refs = { cards: new Set(), crests: new Set() };
    collectNamedRefs(data, refs);
    for (const id of refs.cards) {
      if (!byId.has(id)) fail(file, `dangling card-source id ${id}`);
    }
    for (const id of refs.crests) {
      if (!byId.has(id)) fail(file, `dangling crest id ${id}`);
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
