import type { EngineEvent, PlayerId } from "./types.ts";
import { visual } from "./render/ids.ts";

const STAGGER_MS = 130;
const MAX_PER_HOST = 4;
const floaterTimers: number[] = [];
const liveByHost = new WeakMap<HTMLElement, HTMLElement[]>();

export type CombatTarget = {
  leader?: PlayerId;
  slot?: number;
  player?: PlayerId;
  id?: number;
};

export function targetKey(target: CombatTarget): string {
  if (target.leader) return `leader:${target.leader}`;
  if (typeof target.id === "number" && target.player) return `uid:${target.player}:${target.id}`;
  if (typeof target.slot === "number" && target.player) return `slot:${target.player}:${target.slot}`;
  return "unknown";
}

export function clearFloaters(): void {
  for (const t of floaterTimers) window.clearTimeout(t);
  floaterTimers.length = 0;
  document.querySelectorAll(".floating-combat-text").forEach((el) => el.remove());
}

/** Capture floater anchor rects from the current DOM (before-state) prior to paint. */
export function captureDamageRects(events: EngineEvent[]): Map<string, DOMRect> {
  const rects = new Map<string, DOMRect>();
  for (const ev of events) {
    if ("damage" in ev) {
      const d = ev.damage as { target: CombatTarget; amount: number };
      rememberRect(rects, d.target);
    }
    if ("restore" in ev) {
      const r = ev.restore as { target: CombatTarget; amount: number };
      rememberRect(rects, r.target);
    }
  }
  return rects;
}

function rememberRect(rects: Map<string, DOMRect>, target: CombatTarget): void {
  const key = targetKey(target);
  if (rects.has(key)) return;
  const host = hostFor(target);
  if (host) rects.set(key, host.getBoundingClientRect());
}

export function spawnFloaters(
  events: EngineEvent[],
  enabled: boolean,
  rects?: Map<string, DOMRect>,
): void {
  if (!enabled) return;
  let delay = 0;
  for (const ev of events) {
    if ("damage" in ev) {
      const d = ev.damage as { target: CombatTarget; amount: number };
      const host = hostFor(d.target);
      const rect = rects?.get(targetKey(d.target)) ?? host?.getBoundingClientRect();
      if (rect) {
        queueFloater(rect, "damage", d.amount, delay);
        if (typeof d.target.slot === "number" || typeof d.target.id === "number") {
          if (host) flashCard(host);
        }
      }
      delay += STAGGER_MS;
    }
    if ("restore" in ev) {
      const r = ev.restore as { target: CombatTarget; amount: number };
      const host = hostFor(r.target);
      const rect = rects?.get(targetKey(r.target)) ?? host?.getBoundingClientRect();
      if (rect) queueFloater(rect, "heal", r.amount, delay);
      delay += STAGGER_MS;
    }
  }
}

export function hostFor(target: CombatTarget): HTMLElement | null {
  if (target.leader) {
    return document.getElementById(`${visual(target.leader)}Leader`);
  }
  if (!target.player) return null;
  const board = document.getElementById(`${visual(target.player)}Board`);
  if (!board) return null;
  if (typeof target.id === "number") {
    return board.querySelector<HTMLElement>(`.card[data-uid="${target.id}"]`);
  }
  if (typeof target.slot === "number") {
    return board.querySelector<HTMLElement>(`.card[data-slot="${target.slot}"]`);
  }
  return null;
}

/** Re-apply the 0.45s flash after paint so reconcile cannot strip it. */
export function reflashDamage(events: EngineEvent[]): void {
  for (const ev of events) {
    if (!("damage" in ev)) continue;
    const d = ev.damage as { target: CombatTarget };
    if (typeof d.target.slot !== "number" && typeof d.target.id !== "number") continue;
    const host = hostFor(d.target);
    if (host) flashCard(host);
  }
}

function flashCard(host: HTMLElement): void {
  if (!host.classList.contains("card")) return;
  host.classList.remove("floating-combat-flash");
  void host.offsetWidth;
  host.classList.add("floating-combat-flash");
  window.setTimeout(() => host.classList.remove("floating-combat-flash"), 450);
}

function queueFloater(
  rect: DOMRect,
  kind: "damage" | "heal",
  amount: number,
  delay: number,
): void {
  const show = window.setTimeout(() => {
    const hostKey = `${rect.left}:${rect.top}:${rect.width}`;
    const live = (liveByHost.get(document.body) ?? []).filter((n) => n.isConnected);
    const stacked = live.filter((n) => n.dataset.floaterHost === hostKey);
    if (stacked.length >= MAX_PER_HOST) {
      stacked.shift()?.remove();
    }
    const el = document.createElement("div");
    el.className =
      kind === "damage"
        ? "floating-combat-text floating-combat-text--damage"
        : "floating-combat-text floating-combat-text--heal";
    el.textContent = kind === "damage" ? `-${amount}` : `+${amount}`;
    el.dataset.floaterHost = hostKey;
    el.style.setProperty("--float-stack-index", String(stacked.length));
    el.style.position = "fixed";
    el.style.left = `${rect.left + rect.width / 2}px`;
    el.style.top = `${rect.top + rect.height / 3}px`;
    el.style.transform = "translateX(-50%)";
    el.style.zIndex = "80";
    document.body.appendChild(el);
    stacked.push(el);
    live.push(el);
    liveByHost.set(document.body, live);
    const hide = window.setTimeout(() => {
      el.remove();
      const next = (liveByHost.get(document.body) ?? []).filter((n) => n.isConnected && n !== el);
      liveByHost.set(document.body, next);
    }, 1800);
    floaterTimers.push(hide);
  }, delay);
  floaterTimers.push(show);
}
