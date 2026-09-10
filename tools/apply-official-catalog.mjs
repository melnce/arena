#!/usr/bin/env node
/** Rewrite authored card files' fact fields + text from cards/official/catalog.json. */
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const FACTS = [
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
];

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      if (name === "official") continue;
      walk(p, out);
    } else if (name.endsWith(".json")) out.push(p);
  }
  return out;
}

function rewritePrinted(s) {
  return s
    .replaceAll("Skybound Art-", "Skybound Art -")
    .replaceAll("Super Skybound Art-", "Super Skybound Art -")
    .replaceAll("Enhance(10)", "Enhance (10)");
}

function walkRewrite(node) {
  if (Array.isArray(node)) {
    for (const x of node) walkRewrite(x);
    return;
  }
  if (!node || typeof node !== "object") return;
  if (typeof node.printed === "string") node.printed = rewritePrinted(node.printed);
  for (const v of Object.values(node)) walkRewrite(v);
}

const catalog = JSON.parse(readFileSync(join(ROOT, "cards/official/catalog.json"), "utf8"));
const files = walk(join(ROOT, "cards"));
let n = 0;
for (const file of files) {
  const card = JSON.parse(readFileSync(file, "utf8"));
  if (card.id == null || String(card.id).includes(":")) continue;
  const rec = catalog[String(card.id)];
  if (!rec) {
    console.error(`no catalog record for ${card.id} (${file})`);
    process.exit(1);
  }
  for (const k of FACTS) {
    if ((k === "attack" || k === "defense") && rec.kind !== "follower") continue;
    card[k] = rec[k];
  }
  card.text = rec.text;
  walkRewrite(card);
  writeFileSync(file, JSON.stringify(card, null, 2) + "\n");
  n++;
}
console.log(`updated ${n} card files from catalog`);
