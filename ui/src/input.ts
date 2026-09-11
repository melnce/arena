import {
  attachPointerDragSource,
  shouldSuppressClickFromPointerDrag,
} from "./drag.ts";
import { chooseActionForElement } from "./render.ts";
import type { NeutralAction, PlayerId } from "./types.ts";
import { byId } from "./render/ids.ts";

export type InputHooks = {
  play: (player: PlayerId, handPos: number) => void;
  attack: (player: PlayerId, slot: number, target: { slot: number } | "leader") => void;
  evolve: (player: PlayerId, slot: number, superEvo: boolean) => void;
  engage: (player: PlayerId, slot: number) => void;
  fuse: (player: PlayerId, handPos: number) => void;
  mulliganToggle: (index: number) => void;
  choose: (action: NeutralAction) => void;
  getLegal: () => NeutralAction[];
  getPhase: () => string;
  getActing: () => PlayerId | null;
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

function isLegalPendingTarget(
  pending: NonNullable<ReturnType<InputHooks["getPending"]>>,
  el: HTMLElement | null,
  legal: NeutralAction[],
): boolean {
  if (!el) return false;
  if (pending.kind === "attack") {
    if (el.classList.contains("leader-attack-strip") || el.id.endsWith("Leader")) {
      const target: PlayerId = el.id.startsWith("blue") ? "a" : "b";
      if (target === pending.player) return false;
      return legal.some(
        (a) =>
          "attack" in a &&
          a.attack.player === pending.player &&
          a.attack.attacker_slot === pending.slot &&
          a.attack.target === "leader",
      );
    }
    if (el.closest(".board-zone") && el.dataset.slot != null) {
      const slot = Number(el.dataset.slot);
      const player = el.dataset.player as PlayerId | undefined;
      if (!player || player === pending.player) return false;
      return legal.some(
        (a) =>
          "attack" in a &&
          a.attack.player === pending.player &&
          a.attack.attacker_slot === pending.slot &&
          typeof a.attack.target === "object" &&
          a.attack.target.slot === slot,
      );
    }
    return false;
  }
  if (pending.kind === "evolve") {
    if (!el.closest(".board-zone") || el.dataset.slot == null) return false;
    const slot = Number(el.dataset.slot);
    const player = el.dataset.player as PlayerId | undefined;
    if (player !== pending.player) return false;
    return legal.some(
      (a) =>
        "evolve" in a &&
        a.evolve.player === pending.player &&
        a.evolve.slot === slot &&
        a.evolve.super === pending.superEvo,
    );
  }
  return false;
}

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
    if (t.closest(".fuse-chip") || t.closest(".pending-cancel-chip") || t.closest(".undo-chip")) {
      return;
    }
    if (t.closest("button") && !t.closest(".evo-btn") && !t.closest(".card") && !t.closest(".choice-option")) {
      return;
    }
    const card = t.closest<HTMLElement>(".card");
    if (card && shouldSuppressClickFromPointerDrag(card)) return;

    const legal = hooks.getLegal();
    const phase = hooks.getPhase();
    const pending = hooks.getPending();

    if (phase === "choice") {
      const host = card ?? t.closest<HTMLElement>(".leader-attack-strip");
      if (host) {
        const act = chooseActionForElement(legal, host);
        if (act) hooks.choose(act);
      }
      return;
    }

    const evoBtn = t.closest<HTMLButtonElement>(".evo-btn");
    if (evoBtn && !evoBtn.disabled) {
      const player: PlayerId = evoBtn.id.startsWith("blue") ? "a" : "b";
      const superEvo = evoBtn.id.toLowerCase().includes("super");
      hooks.setPending({ kind: "evolve", player, superEvo });
      return;
    }

    if (pending) {
      const host = card ?? t.closest<HTMLElement>(".leader-attack-strip");
      if (isLegalPendingTarget(pending, host, legal)) {
        if (pending.kind === "attack") {
          if (host?.classList.contains("leader-attack-strip") || host?.id.endsWith("Leader")) {
            hooks.attack(pending.player, pending.slot, "leader");
          } else if (host?.dataset.slot != null) {
            hooks.attack(pending.player, pending.slot, { slot: Number(host.dataset.slot) });
          }
          return;
        }
        if (pending.kind === "evolve" && host?.dataset.slot != null) {
          hooks.evolve(pending.player, Number(host.dataset.slot), pending.superEvo);
          return;
        }
      }
      hooks.cancelPending();
      return;
    }

    if (!card) {
      return;
    }

    const player = card.dataset.player as PlayerId | undefined;
    if (!player) return;

    if (card.closest(".hand-zone")) {
      const pos = Number(card.dataset.handPos);
      if (phase === "mulligan") {
        if (hooks.getActing() === player) hooks.mulliganToggle(pos);
        return;
      }
      if (card.classList.contains("legal-play") || card.classList.contains("playable-glow") || card.classList.contains("enhance-ready")) {
        hooks.play(player, pos);
      }
      return;
    }

    if (card.closest(".board-zone")) {
      const slot = Number(card.dataset.slot);
      if (card.classList.contains("engage-ready")) {
        hooks.engage(player, slot);
        return;
      }
      if (card.classList.contains("can-attack") || card.classList.contains("legal-attack") || card.classList.contains("rush-glow")) {
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
  document.addEventListener("arena-cancel-pending", () => hooks.cancelPending());

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
