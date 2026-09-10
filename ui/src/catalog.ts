import { cardText as wasmCardText } from "../pkg/arena_wasm.js";
import { publicUrl } from "./base.ts";
import type { CardText, CatalogEntry, DeckManifestEntry } from "./types.ts";

const catalog = new Map<string, CatalogEntry>();
const textCache = new Map<string, CardText>();
let deckEntries: DeckManifestEntry[] = [];
let crestBySlug: Record<string, string> = {};
let crestById: Record<string, string> = {};

export async function loadCatalog(): Promise<void> {
  const [images, manifest, crests] = await Promise.all([
    fetch(publicUrl("catalog-images.json")).then((r) => r.json()) as Promise<
      Record<string, CatalogEntry>
    >,
    fetch(publicUrl("decks/manifest.json")).then((r) => r.json()) as Promise<{
      entries: DeckManifestEntry[];
    }>,
    fetch(publicUrl("crest-art.json"))
      .then((r) => r.json())
      .catch(() => ({ bySlug: {}, byId: {} })) as Promise<{
      bySlug?: Record<string, string>;
      byId?: Record<string, string>;
    }>,
  ]);
  catalog.clear();
  for (const [id, rec] of Object.entries(images)) catalog.set(id, rec);
  deckEntries = manifest.entries ?? [];
  crestBySlug = crests.bySlug ?? {};
  crestById = crests.byId ?? {};
}

export function getCatalog(id: string): CatalogEntry | undefined {
  return catalog.get(id);
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

export function crestFile(id: string, name?: string): string | null {
  if (crestById[id]) return crestById[id];
  const slug = String(name || id)
    .toLowerCase()
    .replace(/^crest:\s*/i, "")
    .replace(/['’]/g, "")
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_|_$/g, "");
  if (crestBySlug[slug]) return crestBySlug[slug];
  const first = slug.split("_")[0];
  if (first && crestBySlug[first]) return crestBySlug[first];
  return null;
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
