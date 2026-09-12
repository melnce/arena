#!/usr/bin/env node
// Emit ui/public/catalog-images.json + copy oracle decks into ui/public/decks/.
// Image hashes stay in JSON (not WASM) so a catalog refresh does not rebuild wasm.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..");
const catalogPath = path.join(repo, "cards/official/catalog.json");
const decksSrc = path.join(repo, "oracle/decks");
const outPublic = path.join(repo, "ui/public");
const outCatalog = path.join(outPublic, "catalog-images.json");
const outDecks = path.join(outPublic, "decks");

function slug(name) {
  return String(name || "")
    .toLowerCase()
    .replace(/^crest:\s*/i, "")
    .replace(/['’]/g, "")
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_|_$/g, "");
}

const raw = JSON.parse(fs.readFileSync(catalogPath, "utf8"));
const images = {};
for (const [id, rec] of Object.entries(raw)) {
  if (!rec || typeof rec !== "object" || id.startsWith("_")) continue;
  if (!rec.id && !rec.card_image_hash && !rec.name) continue;
  images[id] = {
    name: rec.name ?? id,
    cost: rec.cost ?? null,
    class: rec.class ?? null,
    kind: rec.kind ?? null,
    attack: rec.attack ?? null,
    defense: rec.defense ?? null,
    card: rec.card_image_hash ?? "",
    banner: rec.card_banner_image_hash ?? "",
    evoCard: rec.evo_card_image_hash ?? "",
    evoBanner: rec.evo_card_banner_image_hash ?? "",
  };
}

fs.mkdirSync(outPublic, { recursive: true });
fs.writeFileSync(outCatalog, JSON.stringify(images));

fs.mkdirSync(outDecks, { recursive: true });
const files = fs
  .readdirSync(decksSrc)
  .filter((f) => f.endsWith(".json"))
  .sort();
const entries = [];
for (const file of files) {
  fs.copyFileSync(path.join(decksSrc, file), path.join(outDecks, file));
  const id = file.replace(/\.json$/i, "");
  entries.push({
    file,
    id,
    label: id,
    category: "deck",
  });
}
fs.writeFileSync(
  path.join(outDecks, "manifest.json"),
  JSON.stringify({ entries }, null, 2) + "\n",
);

const crestDir = path.join(outPublic, "crests");
const crestFiles = fs.existsSync(crestDir)
  ? fs.readdirSync(crestDir).filter((f) => f.endsWith(".png"))
  : [];
const bySlug = {};
for (const f of crestFiles) bySlug[slug(f.replace(/\.png$/i, ""))] = f;
const crestMap = {};
for (const [id, rec] of Object.entries(images)) {
  const s = slug(rec.name);
  if (bySlug[s]) crestMap[id] = bySlug[s];
}

fs.writeFileSync(
  path.join(outPublic, "crest-art.json"),
  JSON.stringify({ bySlug, byId: crestMap }, null, 2) + "\n",
);

console.log(
  `catalog-images ${Object.keys(images).length} cards; decks ${files.length}; crests ${crestFiles.length}`,
);
