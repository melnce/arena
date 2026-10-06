import { lookupText } from "./catalog.ts";
import { applyCardImage, cardImageUrl } from "./images.ts";
import { visual } from "./render/ids.ts";
import type { EngineEvent, PlayerId } from "./types.ts";

export type PlayForm =
  | "normal"
  | { enhance: number }
  | { accelerate: number }
  | { crystallize: number };

export type SpotlightLogEntry = {
  card: string;
  form: PlayForm;
  shownAt: number;
  hiddenAt: number;
};

type QueuedPlay = {
  card: string;
  form: PlayForm;
};

const DISPLAY_MS = 1500;
const MIN_VISIBLE_MS = 1200;
const FADE_IN_MS = 150;
const FADE_OUT_MS = 200;
const MAX_QUEUE = 3;

let log: SpotlightLogEntry[] = [];
let queue: QueuedPlay[] = [];
let pump = 0;
let busy = false;
let humanSide: PlayerId = "a";
let currentEntry: SpotlightLogEntry | null = null;
let shownAt = 0;
let fadeTimer = 0;

export function botPlaySpotlightOn(): boolean {
  const box = document.getElementById("botPlaySpotlightToggle") as HTMLInputElement | null;
  if (box) return box.checked;
  try {
    const v = localStorage.getItem("svwb.botPlaySpotlight");
    return v == null ? true : v !== "0";
  } catch {
    return true;
  }
}

export function spotlightLog(): SpotlightLogEntry[] {
  return log.slice();
}

export function resetSpotlightSession(): void {
  log = [];
  clearBotSpotlight();
}

export function clearBotSpotlight(): void {
  window.clearTimeout(fadeTimer);
  queue = [];
  pump += 1;
  busy = false;
  finishCurrentEntry();
  const el = document.getElementById("botPlaySpotlight");
  if (el) {
    el.classList.remove("visible", "hiding");
    el.innerHTML = "";
    el.setAttribute("aria-hidden", "true");
  }
}

function finishCurrentEntry(): void {
  if (currentEntry && currentEntry.hiddenAt === 0) {
    currentEntry.hiddenAt = performance.now();
  }
  currentEntry = null;
  shownAt = 0;
}

function parseForm(form: unknown): PlayForm {
  if (form === "normal" || form == null) return "normal";
  if (typeof form === "object" && form !== null) {
    const o = form as Record<string, unknown>;
    if ("enhance" in o) return { enhance: Number(o.enhance) };
    if ("accelerate" in o) return { accelerate: Number(o.accelerate) };
    if ("crystallize" in o) return { crystallize: Number(o.crystallize) };
  }
  return "normal";
}

function formChipLabel(form: PlayForm): string | null {
  if (form === "normal") return null;
  if ("enhance" in form) return "Enhance";
  if ("accelerate" in form) return "Accelerate";
  if ("crystallize" in form) return "Crystallize";
  return null;
}

function extractBotPlays(events: EngineEvent[], side: PlayerId): QueuedPlay[] {
  const out: QueuedPlay[] = [];
  for (const ev of events) {
    const play = ev.play as { player?: string; card?: string; form?: unknown } | undefined;
    if (!play?.card || play.player === side) continue;
    out.push({ card: play.card, form: parseForm(play.form) });
  }
  return out;
}

export function enqueueBotSpotlights(events: EngineEvent[], cfgHumanSide: PlayerId): void {
  if (!botPlaySpotlightOn()) return;
  humanSide = cfgHumanSide;
  const plays = extractBotPlays(events, cfgHumanSide);
  if (!plays.length) return;
  for (const p of plays) {
    if (queue.length >= MAX_QUEUE) queue.shift();
    queue.push(p);
  }
  void drainQueue();
}

function waitMs(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

async function drainQueue(): Promise<void> {
  if (busy) return;
  const token = pump;
  busy = true;
  try {
    while (queue.length > 0) {
      if (token !== pump) return;
      const next = queue.shift()!;
      if (currentEntry) {
        const elapsed = performance.now() - shownAt;
        if (elapsed < MIN_VISIBLE_MS) {
          await waitMs(MIN_VISIBLE_MS - elapsed);
          if (token !== pump) return;
        }
        await hideSpotlight(token);
        if (token !== pump) return;
      }
      await showSpotlight(next);
      if (token !== pump) return;
      const deadline = shownAt + DISPLAY_MS;
      while (performance.now() < deadline) {
        if (token !== pump) return;
        if (queue.length > 0 && performance.now() - shownAt >= MIN_VISIBLE_MS) break;
        await waitMs(40);
      }
      if (token !== pump) return;
      await hideSpotlight(token);
      if (token !== pump) return;
    }
  } finally {
    if (token === pump) {
      busy = false;
      if (queue.length > 0) void drainQueue();
    }
  }
}

function ensureOverlay(): HTMLElement {
  let el = document.getElementById("botPlaySpotlight");
  if (!el) {
    el = document.createElement("div");
    el.id = "botPlaySpotlight";
    el.setAttribute("aria-hidden", "true");
    document.body.appendChild(el);
  }
  return el;
}

function rectsOverlap(a: DOMRect, b: DOMRect, pad = 2): boolean {
  return !(
    a.right + pad < b.left ||
    a.left - pad > b.right ||
    a.bottom + pad < b.top ||
    a.top - pad > b.bottom
  );
}

function collectRects(selector: string): DOMRect[] {
  const out: DOMRect[] = [];
  for (const el of document.querySelectorAll<HTMLElement>(selector)) {
    const r = el.getBoundingClientRect();
    if (r.width > 0 && r.height > 0) out.push(r);
  }
  return out;
}

function forbiddenRects(): DOMRect[] {
  const humanHand = humanSide === "a" ? "#blueHand" : "#redHand";
  const rects = [
    ...collectRects("#redBoard .card"),
    ...collectRects("#blueBoard .card"),
    ...collectRects(`${humanHand} .card`),
    ...collectRects("#redLeader"),
    ...collectRects("#blueLeader"),
  ];
  const rail = document.getElementById("turnControls");
  if (rail) {
    const r = rail.getBoundingClientRect();
    if (r.width > 0 && r.height > 0) rects.push(r);
  }
  return rects;
}

function clampRect(left: number, top: number, w: number, h: number): DOMRect {
  const margin = 8;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  let x = left;
  let y = top;
  if (x < margin) x = margin;
  if (y < margin) y = margin;
  if (x + w > vw - margin) x = Math.max(margin, vw - margin - w);
  if (y + h > vh - margin) y = Math.max(margin, vh - margin - h);
  return new DOMRect(x, y, w, h);
}

function candidateRect(handRect: DOMRect, maxW: number, maxH: number, scale: number): DOMRect {
  const w = maxW * scale;
  const h = maxH * scale;
  const left = handRect.left + Math.min(handRect.width * 0.08, 16);
  const top = handRect.top + (handRect.height - h) / 2;
  return clampRect(left, top, w, h);
}

function fitsSpotlight(rect: DOMRect, blocked: DOMRect[]): boolean {
  return !blocked.some((b) => rectsOverlap(rect, b));
}

function applySpotlightRect(overlay: HTMLElement, rect: DOMRect): void {
  overlay.style.left = `${rect.left}px`;
  overlay.style.top = `${rect.top}px`;
  overlay.style.width = `${rect.width}px`;
  overlay.style.height = `${rect.height}px`;
}

function layoutSpotlight(botPlayer: PlayerId): boolean {
  const hand = document.getElementById(visual(botPlayer) === "blue" ? "blueHand" : "redHand");
  const overlay = ensureOverlay();
  const ref = hand?.querySelector<HTMLElement>(".card");
  const refRect = ref?.getBoundingClientRect();
  const cssH = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--card-height"));
  const baseH = refRect?.height || cssH || 150;
  const baseW = refRect?.width || baseH * (2 / 3);
  const maxW = baseW * 1.3;
  const maxH = baseH * 1.3;
  const handRect = hand?.getBoundingClientRect() ?? new DOMRect(16, 16, maxW * 2, maxH);
  const blocked = forbiddenRects();
  let smallest = candidateRect(handRect, maxW, maxH, 7 / 20);
  let chosen: DOMRect | null = null;

  for (let i = 20; i >= 7; i--) {
    const scale = i / 20;
    const candidate = candidateRect(handRect, maxW, maxH, scale);
    smallest = candidate;
    if (fitsSpotlight(candidate, blocked)) {
      chosen = candidate;
      break;
    }
  }

  const rect = chosen ?? smallest;
  if (!fitsSpotlight(rect, blocked)) return false;
  applySpotlightRect(overlay, rect);
  return true;
}

function renderSpotlightCard(play: QueuedPlay): void {
  const overlay = ensureOverlay();
  overlay.innerHTML = "";
  const card = document.createElement("div");
  card.className = "spotlight-card";
  card.dataset.card = play.card;

  const art = document.createElement("div");
  art.className = "spotlight-art";
  const img = document.createElement("img");
  img.className = "spotlight-img";
  img.alt = lookupText(play.card).name;
  applyCardImage(img, play.card, false, art);
  if (cardImageUrl(play.card, false)) {
    img.dataset.expectedSrc = cardImageUrl(play.card, false)!;
  }
  art.appendChild(img);
  card.appendChild(art);

  const name = document.createElement("div");
  name.className = "spotlight-name";
  name.textContent = lookupText(play.card).name;
  card.appendChild(name);

  const chip = formChipLabel(play.form);
  if (chip) {
    const badge = document.createElement("span");
    badge.className = "spotlight-form-chip";
    badge.textContent = chip;
    card.appendChild(badge);
  }

  overlay.appendChild(card);
}

async function showSpotlight(play: QueuedPlay): Promise<void> {
  const botPlayer: PlayerId = humanSide === "a" ? "b" : "a";
  const now = performance.now();
  currentEntry = { card: play.card, form: play.form, shownAt: now, hiddenAt: 0 };
  log.push(currentEntry);
  if (!layoutSpotlight(botPlayer)) {
    currentEntry.hiddenAt = now;
    finishCurrentEntry();
    return;
  }
  const overlay = ensureOverlay();
  renderSpotlightCard(play);
  overlay.setAttribute("aria-hidden", "false");
  overlay.classList.remove("hiding");
  void overlay.offsetWidth;
  overlay.classList.add("visible");
  shownAt = now;
  currentEntry.shownAt = now;
  await waitMs(FADE_IN_MS);
}

async function hideSpotlight(token: number): Promise<void> {
  const overlay = document.getElementById("botPlaySpotlight");
  if (!overlay || !overlay.classList.contains("visible")) {
    if (token === pump) finishCurrentEntry();
    return;
  }
  overlay.classList.add("hiding");
  overlay.classList.remove("visible");
  if (token === pump) finishCurrentEntry();
  await waitMs(FADE_OUT_MS);
  if (token !== pump) return;
  overlay.classList.remove("hiding");
  overlay.innerHTML = "";
  overlay.setAttribute("aria-hidden", "true");
}
