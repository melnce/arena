import { cardText as wasmCardText } from "../pkg/arena_wasm.js";
import { publicUrl } from "./base.ts";
import type { CardText, CatalogEntry, DeckManifestEntry } from "./types.ts";

const catalog = new Map<string, CatalogEntry>();
const textCache = new Map<string, CardText>();
let deckEntries: DeckManifestEntry[] = [];

export async function loadCatalog(): Promise<void> {
  const [images, manifest] = await Promise.all([
    fetch(publicUrl("catalog-images.json")).then((r) => r.json()) as Promise<
      Record<string, CatalogEntry>
    >,
    fetch(publicUrl("decks/manifest.json")).then((r) => r.json()) as Promise<{
      entries: DeckManifestEntry[];
    }>,
  ]);
  catalog.clear();
  for (const [id, rec] of Object.entries(images)) catalog.set(id, rec);
  deckEntries = manifest.entries ?? [];
}

export function getCatalog(id: string): CatalogEntry | undefined {
  return catalog.get(id);
}

export function catalogIds(): string[] {
  return [...catalog.keys()];
}

export function lookupText(id: string): CardText {
  const hit = textCache.get(id);
  if (hit) return hit;
  try {
    const info = JSON.parse(wasmCardText(id)) as CardText;
    textCache.set(id, info);
    return info;
  } catch {
    const cat = catalog.get(id);
    const fallback: CardText = {
      id,
      name: cat?.name ?? id,
      text: "",
      kind: cat?.kind ?? "?",
      cost: cat?.cost ?? null,
    };
    textCache.set(id, fallback);
    return fallback;
  }
}

export function decks(): DeckManifestEntry[] {
  return deckEntries;
}

export function parseDeckJson(raw: string): Record<string, number> {
  const v = JSON.parse(raw) as unknown;
  if (Array.isArray(v)) {
    const out: Record<string, number> = {};
    for (const id of v) {
      const key = String(id);
      out[key] = (out[key] ?? 0) + 1;
    }
    return out;
  }
  if (v && typeof v === "object") {
    const out: Record<string, number> = {};
    for (const [k, n] of Object.entries(v as Record<string, unknown>)) {
      const count = Number(n);
      if (!Number.isFinite(count) || count <= 0) continue;
      out[k] = count;
    }
    return out;
  }
  throw new Error("deck JSON must be {id: count} or [ids]");
}
