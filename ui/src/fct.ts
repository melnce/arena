import type { EngineEvent, PlayerId } from "./types.ts";
import { visual } from "./render/ids.ts";

const STAGGER_MS = 130;
const floaterTimers: number[] = [];

export function clearFloaters(): void {
  for (const t of floaterTimers) window.clearTimeout(t);
  floaterTimers.length = 0;
  document.querySelectorAll(".floating-combat-text").forEach((el) => el.remove());
}

export function spawnFloaters(events: EngineEvent[], enabled: boolean): void {
  if (!enabled) return;
  let delay = 0;
  for (const ev of events) {
    if ("damage" in ev) {
      const d = ev.damage as {
        target: { leader?: PlayerId; slot?: number; player?: PlayerId };
        amount: number;
      };
      const host = hostFor(d.target);
      if (host) queueFloater(host, "damage", d.amount, delay);
      delay += STAGGER_MS;
    }
    if ("restore" in ev) {
      const r = ev.restore as {
        target: { leader?: PlayerId; slot?: number; player?: PlayerId };
        amount: number;
      };
      const host = hostFor(r.target);
      if (host) queueFloater(host, "heal", r.amount, delay);
      delay += STAGGER_MS;
    }
  }
}

function hostFor(target: {
  leader?: PlayerId;
  slot?: number;
  player?: PlayerId;
}): HTMLElement | null {
  if (target.leader) {
    return document.getElementById(`${visual(target.leader)}Leader`);
  }
  if (typeof target.slot === "number" && target.player) {
    const board = document.getElementById(`${visual(target.player)}Board`);
    return board?.querySelector(`.card[data-slot="${target.slot}"]`) ?? null;
  }
  return null;
}

function queueFloater(
  host: HTMLElement,
  kind: "damage" | "heal",
  amount: number,
  delay: number,
): void {
  const rect = host.getBoundingClientRect();
  const show = window.setTimeout(() => {
    const el = document.createElement("div");
    el.className =
      kind === "damage"
        ? "floating-combat-text floating-combat-text--damage"
        : "floating-combat-text floating-combat-text--heal";
    el.textContent = kind === "damage" ? `-${amount}` : `+${amount}`;
    el.style.position = "fixed";
    el.style.left = `${rect.left + rect.width / 2}px`;
    el.style.top = `${rect.top + rect.height / 3}px`;
    el.style.transform = "translateX(-50%)";
    el.style.zIndex = "80";
    document.body.appendChild(el);
    const hide = window.setTimeout(() => el.remove(), 1800);
    floaterTimers.push(hide);
  }, delay);
  floaterTimers.push(show);
}
