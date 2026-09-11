import {
  attachPointerDragSource,
  shouldSuppressClickFromPointerDrag,
} from "./drag.ts";
import type { PlayerId } from "./types.ts";
import { byId } from "./render/ids.ts";

export type InputHooks = {
  play: (player: PlayerId, handPos: number) => void;
  attack: (player: PlayerId, slot: number, target: { slot: number } | "leader") => void;
  evolve: (player: PlayerId, slot: number, superEvo: boolean) => void;
  engage: (player: PlayerId, slot: number) => void;
  fuse: (player: PlayerId, handPos: number) => void;
  mulliganToggle: (index: number) => void;
  getPending: () =>
    | { kind: "attack"; player: PlayerId; slot: number }
    | { kind: "evolve"; player: PlayerId; superEvo: boolean }
    | { kind: "play"; player: PlayerId; handPos: number }
    | null;
  setPending: (
    p:
      | { kind: "attack"; player: PlayerId; slot: number }
      | { kind: "evolve"; player: PlayerId; superEvo: boolean }
      | { kind: "play"; player: PlayerId; handPos: number }
      | null,
  ) => void;
  cancelPending: () => void;
};

export function bindPointer(hooks: InputHooks): void {
  document.addEventListener(
    "pointerdown",
    (e) => {
      const t = e.target as HTMLElement;
      const card = t.closest<HTMLElement>(".card");
      if (card) card.classList.add("pressed");
    },
    true,
  );
  document.addEventListener(
    "pointerup",
    () => {
      document.querySelectorAll(".card.pressed").forEach((n) => n.classList.remove("pressed"));
    },
    true,
  );

  document.addEventListener("click", (e) => {
    const t = e.target as HTMLElement;
    if (t.closest("#settingsDrawer") || t.closest("#settingsToggle")) return;
    if (t.closest("button") && !t.closest(".evo-btn") && !t.closest(".card")) return;
    const card = t.closest<HTMLElement>(".card");
    if (card && shouldSuppressClickFromPointerDrag(card)) return;

    const pending = hooks.getPending();
    const evoBtn = t.closest<HTMLButtonElement>(".evo-btn");
    if (evoBtn && !evoBtn.disabled) {
      const player: PlayerId = evoBtn.id.startsWith("blue") ? "a" : "b";
      const superEvo = evoBtn.id.toLowerCase().includes("super");
      hooks.setPending({ kind: "evolve", player, superEvo });
      return;
    }

    if (!card) {
      const leader = t.closest<HTMLElement>(".leader-attack-strip");
      if (leader && pending?.kind === "attack") {
        const target: PlayerId = leader.id.startsWith("blue") ? "a" : "b";
        if (target !== pending.player) hooks.attack(pending.player, pending.slot, "leader");
      }
      return;
    }

    const player = card.dataset.player as PlayerId | undefined;
    if (!player) return;

    if (card.closest(".hand-zone")) {
      const phase = byId("turnCounter")?.dataset.phase;
      const pos = Number(card.dataset.handPos);
      if (phase === "mulligan") {
        hooks.mulliganToggle(pos);
        return;
      }
      if (card.classList.contains("legal-play") || card.classList.contains("playable-glow")) {
        hooks.play(player, pos);
      }
      return;
    }

    if (card.closest(".board-zone")) {
      const slot = Number(card.dataset.slot);
      if (pending?.kind === "evolve" && pending.player === player) {
        hooks.evolve(player, slot, pending.superEvo);
        return;
      }
      if (pending?.kind === "attack" && pending.player !== player) {
        hooks.attack(pending.player, pending.slot, { slot });
        return;
      }
      if (card.classList.contains("engage-ready")) {
        hooks.engage(player, slot);
        return;
      }
      if (card.classList.contains("can-attack") || card.classList.contains("legal-attack")) {
        hooks.setPending({ kind: "attack", player, slot });
      }
    }
  });

  document.addEventListener("contextmenu", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLElement>(".card");
    if (!card?.classList.contains("fuse-ready")) return;
    e.preventDefault();
    const player = card.dataset.player as PlayerId;
    hooks.fuse(player, Number(card.dataset.handPos));
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") hooks.cancelPending();
  });

  for (const id of ["blueNormalEvo", "blueSuperEvo", "redNormalEvo", "redSuperEvo"]) {
    const btn = byId<HTMLButtonElement>(id);
    if (!btn) continue;
    const player: PlayerId = id.startsWith("blue") ? "a" : "b";
    const superEvo = id.toLowerCase().includes("super");
    attachPointerDragSource(
      btn,
      { payload: JSON.stringify({ kind: "evo", player, superEvo }), kind: "evo" },
      true,
    );
  }
}
