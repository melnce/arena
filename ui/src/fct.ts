import type { EngineEvent, PlayerId } from "./types.ts";
import { visual } from "./render/ids.ts";

const STAGGER_MS = 130;

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
    return document.getElementById(`${visual(target.player)}-board-${target.slot}`);
  }
  return null;
}

function queueFloater(
  host: HTMLElement,
  kind: "damage" | "heal",
  amount: number,
  delay: number,
): void {
  window.setTimeout(() => {
    const el = document.createElement("div");
    el.className =
      kind === "damage"
        ? "floating-combat-text floating-combat-text--damage"
        : "floating-combat-text floating-combat-text--heal";
    el.textContent = kind === "damage" ? `-${amount}` : `+${amount}`;
    host.appendChild(el);
    window.setTimeout(() => el.remove(), 1800);
  }, delay);
}
