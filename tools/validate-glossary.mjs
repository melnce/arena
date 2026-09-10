#!/usr/bin/env node
/**
 * Regenerate rules/official-glossary.md from committed glossary JSON and
 * fail if the markdown differs (same contract as gen_schema.py for the schema).
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { officialGlossaryMarkdown } from "./fetch-glossary.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const EN_OUT = path.join(ROOT, "cards", "official", "glossary.en.json");
const JA_OUT = path.join(ROOT, "cards", "official", "glossary.ja.json");
const MD_OUT = path.join(ROOT, "rules", "official-glossary.md");

function main() {
  if (!fs.existsSync(EN_OUT) || !fs.existsSync(JA_OUT)) {
    console.error("missing committed glossary JSON");
    process.exit(1);
  }
  if (!fs.existsSync(MD_OUT)) {
    console.error(`missing ${MD_OUT}`);
    process.exit(1);
  }
  const en = JSON.parse(fs.readFileSync(EN_OUT, "utf8"));
  const ja = JSON.parse(fs.readFileSync(JA_OUT, "utf8"));
  const expected = officialGlossaryMarkdown(en, ja);
  const committed = fs.readFileSync(MD_OUT, "utf8");
  if (expected !== committed) {
    console.error(
      "rules/official-glossary.md does not match regeneration from cards/official/glossary.*.json",
    );
    console.error("run: node tools/fetch-glossary.mjs --md-from-json");
    process.exit(1);
  }
  console.log(`ok: official glossary markdown matches JSON (${en._meta?.count ?? "?"} entries)`);
}

main();
