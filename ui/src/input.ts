import * as L from "./legal.ts";
import type { NeutralAction, PlayerId } from "./types.ts";

export type DragHooks = {
  legal: () => NeutralAction[];
  play: (player: PlayerId, handPos: number) => void;
  attack: (player: PlayerId, slot: number, target: { slot: number } | "leader") => void;
  evolve: (player: PlayerId, slot: number, superEvo: boolean) => void;
  getEvoArmed: () => { player: PlayerId; superEvo: boolean } | null;
  clearEvoArmed: () => void;
};

type DragKind =
  | { kind: "play"; player: PlayerId; handPos: number; x: number; y: number }
  | { kind: "attack"; player: PlayerId; slot: number; x: number; y: number }
  | { kind: "evo"; player: PlayerId; superEvo: boolean; x: number; y: number };

let drag: DragKind | null = null;
let ghost: HTMLElement | null = null;

export function bindPointer(hooks: DragHooks): void {
  document.addEventListener("pointerdown", (e) => {
    const t = e.target as HTMLElement;
    const evoBtn = t.closest<HTMLButtonElement>(".evo-btn");
    if (evoBtn && !evoBtn.disabled) {
      const id = evoBtn.id;
      const player: PlayerId = id.startsWith("blue") ? "a" : "b";
      const superEvo = id.toLowerCase().includes("super");
      drag = { kind: "evo", player, superEvo, x: e.clientX, y: e.clientY };
      startGhost(evoBtn, e);
      return;
    }
    const card = t.closest<HTMLElement>(".card");
    if (!card) return;
    const player = card.dataset.player as PlayerId | undefined;
    if (!player) return;
    if (card.closest(".hand-zone") && card.classList.contains("legal-play")) {
      drag = {
        kind: "play",
        player,
        handPos: Number(card.dataset.handPos),
        x: e.clientX,
        y: e.clientY,
      };
      startGhost(card, e);
      return;
    }
    if (card.closest(".board-zone") && card.classList.contains("can-attack")) {
      drag = { kind: "attack", player, slot: Number(card.dataset.slot), x: e.clientX, y: e.clientY };
      startGhost(card, e);
    }
  });

  document.addEventListener("pointermove", (e) => {
    if (!ghost) return;
    ghost.style.left = `${e.clientX}px`;
    ghost.style.top = `${e.clientY}px`;
    highlightDrop(e.clientX, e.clientY, hooks.legal());
  });

  document.addEventListener("pointerup", (e) => {
    if (!drag) {
      const armed = hooks.getEvoArmed();
      if (armed) {
        const card = (e.target as HTMLElement).closest<HTMLElement>(".card");
        if (card?.dataset.slot != null && card.dataset.player === armed.player) {
          hooks.evolve(armed.player, Number(card.dataset.slot), armed.superEvo);
        }
        hooks.clearEvoArmed();
      }
      return;
    }
    const kind = drag;
    const x = e.clientX;
    const y = e.clientY;
    clearGhost();
    drag = null;
    const el = document.elementFromPoint(x, y) as HTMLElement | null;
    if (!el) return;
    if (kind.kind === "play") {
      const moved = Math.hypot(x - kind.x, y - kind.y);
      const onBoard = !!(el.closest(".board-zone") || el.closest(".leader"));
      if (moved < 10 || onBoard || el.closest(".hand-zone") || el.closest("#appRoot")) {
        hooks.play(kind.player, kind.handPos);
      }
      return;
    }
    if (kind.kind === "attack") {
      const leader = el.closest<HTMLElement>(".leader-attack-strip");
      if (leader) {
        const target: PlayerId = leader.id.startsWith("blue") ? "a" : "b";
        if (target !== kind.player) hooks.attack(kind.player, kind.slot, "leader");
        return;
      }
      const card = el.closest<HTMLElement>(".card");
      if (card?.dataset.slot != null && card.dataset.player && card.dataset.player !== kind.player) {
        hooks.attack(kind.player, kind.slot, { slot: Number(card.dataset.slot) });
      }
      return;
    }
    if (kind.kind === "evo") {
      const card = el.closest<HTMLElement>(".card");
      if (card?.dataset.slot != null && card.dataset.player === kind.player) {
        hooks.evolve(kind.player, Number(card.dataset.slot), kind.superEvo);
      }
    }
  });
}

function startGhost(from: HTMLElement, e: PointerEvent): void {
  clearGhost();
  ghost = from.cloneNode(true) as HTMLElement;
  ghost.classList.add("drag-ghost");
  ghost.style.left = `${e.clientX}px`;
  ghost.style.top = `${e.clientY}px`;
  document.body.appendChild(ghost);
}

function clearGhost(): void {
  ghost?.remove();
  ghost = null;
  document.querySelectorAll(".drop-hover").forEach((n) => n.classList.remove("drop-hover"));
}

function highlightDrop(x: number, y: number, legal: NeutralAction[]): void {
  document.querySelectorAll(".drop-hover").forEach((n) => n.classList.remove("drop-hover"));
  const el = document.elementFromPoint(x, y) as HTMLElement | null;
  if (!el || !drag) return;
  if (drag.kind === "attack") {
    const acts = L.attacksFrom(legal, drag.player, drag.slot);
    const leader = el.closest<HTMLElement>(".leader-attack-strip");
    if (leader && acts.some((a) => "attack" in a && a.attack.target === "leader")) {
      leader.classList.add("drop-hover", "legal-target");
    }
    const card = el.closest<HTMLElement>(".card");
    if (card?.dataset.slot != null) {
      const slot = Number(card.dataset.slot);
      if (acts.some((a) => "attack" in a && typeof a.attack.target === "object" && a.attack.target.slot === slot)) {
        card.classList.add("drop-hover", "legal-target");
      }
    }
  }
}
