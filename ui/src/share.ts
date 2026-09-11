import type { Mode } from "./types.ts";

export type ShareParams = {
  seed?: string;
  deckA?: string;
  deckB?: string;
  mode?: Mode;
};

export function readShareParams(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): ShareParams {
  const p = new URLSearchParams(search);
  const out: ShareParams = {};
  const seed = p.get("seed");
  if (seed) out.seed = seed;
  const a = p.get("deckA") ?? p.get("a");
  const b = p.get("deckB") ?? p.get("b");
  if (a) out.deckA = a;
  if (b) out.deckB = b;
  const mode = p.get("mode");
  if (mode === "hotseat" || mode === "vs-bot" || mode === "watch") out.mode = mode;
  return out;
}

export function writeShareParams(params: {
  seed: string;
  deckA: string;
  deckB: string;
  mode: Mode;
}): void {
  if (typeof window === "undefined" || typeof history === "undefined") return;
  const url = new URL(window.location.href);
  url.searchParams.set("seed", params.seed);
  url.searchParams.set("deckA", params.deckA);
  url.searchParams.set("deckB", params.deckB);
  url.searchParams.set("mode", params.mode);
  url.searchParams.set("a", params.deckA);
  url.searchParams.set("b", params.deckB);
  history.replaceState(null, "", `${url.pathname}${url.search}${url.hash}`);
}
