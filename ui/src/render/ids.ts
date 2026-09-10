import type { PlayerId } from "../types.ts";

export function visual(p: PlayerId): "blue" | "red" {
  return p === "a" ? "blue" : "red";
}

export function byId<T extends HTMLElement>(id: string): T | null {
  return document.getElementById(id) as T | null;
}

export function setText(id: string, text: string | number): void {
  const el = byId(id);
  if (el) el.textContent = String(text);
}
