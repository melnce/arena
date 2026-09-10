import { lookupText } from "./catalog.ts";
import { cardImageUrl, escapeHtml } from "./images.ts";
import * as L from "./legal.ts";
import type { Session } from "./session.ts";
import { fullState, legalActions } from "./session.ts";
import type {
  ChoiceNode,
  FullState,
  NeutralAction,
  PlayerId,
  TargetOpt,
} from "./types.ts";
import { cardSignature, renderCard, renderCrestSlot } from "./render/card.ts";
import { byId, setText, visual } from "./render/ids.ts";

export type RenderHooks = {
  onPlay: (player: PlayerId, handPos: number) => void;
  onFuse: (player: PlayerId, handPos: number) => void;
  onEngage: (player: PlayerId, slot: number) => void;
  onAttack: (player: PlayerId, slot: number, target: { slot: number } | "leader") => void;
  onEvolve: (player: PlayerId, slot: number, superEvo: boolean) => void;
  onChooseOpt: (action: NeutralAction) => void;
  onConfirm: () => void;
  onMulliganToggle: (index: number) => void;
  onMulliganConfirm: (player: PlayerId) => void;
  onEndTurn: (player: PlayerId) => void;
  onBonusPp: (player: PlayerId) => void;
  onNewGame: () => void;
  onRematchSwap: () => void;
  evoArmed: { player: PlayerId; superEvo: boolean } | null;
};

const zoneSig = new Map<string, string>();

export function render(s: Session, hooks: RenderHooks): void {
  const full = fullState(s);
  const legal = legalActions(s);
  const acting = s.game.acting() as PlayerId;
  const phase = s.game.phase();

  document.body.classList.toggle("active-first", full.active === "a");
  document.body.classList.toggle("active-second", full.active === "b");
  document.body.classList.toggle("gameover", phase === "terminal");
  document.body.classList.toggle("select-mode", phase === "choice");
  document.body.classList.toggle("mode-watch", s.cfg.mode === "watch");
  document.body.classList.toggle("mode-vs-bot", s.cfg.mode === "vs-bot");
  document.body.classList.toggle("mode-hotseat", s.cfg.mode === "hotseat");

  setText("blueHP", full.players.a.leader_defense);
  setText("redHP", full.players.b.leader_defense);
  setText("bluePP", `${full.players.a.pp}/${full.players.a.pp_max}`);
  setText("redPP", `${full.players.b.pp}/${full.players.b.pp_max}`);
  setText("blueShadows", full.players.a.shadows);
  setText("redShadows", full.players.b.shadows);
  setText("blueHandCount", full.players.a.hand.length);
  setText("redHandCount", full.players.b.hand.length);
  setText("blueDeckCount", full.players.a.deck.length);
  setText("redDeckCount", full.players.b.deck.length);
  setText("blueGraveCount", full.players.a.cemetery.length);
  setText("redGraveCount", full.players.b.cemetery.length);

  const turnEl = byId("turnCounter");
  if (turnEl) {
    turnEl.textContent = String(s.ply);
    turnEl.dataset.turn = String(full.turn);
    turnEl.dataset.active = full.active;
    turnEl.dataset.phase = phase;
    turnEl.dataset.acting = acting;
  }
  setText(
    "turnReadout",
    phase === "terminal"
      ? `Winner ${full.winner ?? "?"}`
      : `Turn ${full.turn || 0} · ${acting.toUpperCase()}`,
  );

  const hideA = s.cfg.mode === "vs-bot" && s.cfg.hideBotHand && s.cfg.humanSide !== "a";
  const hideB = s.cfg.mode === "vs-bot" && s.cfg.hideBotHand && s.cfg.humanSide !== "b";

  renderHand(s, full, legal, "a", hideA, hooks);
  renderHand(s, full, legal, "b", hideB, hooks);
  renderBoard(full, legal, "a", hooks);
  renderBoard(full, legal, "b", hooks);
  renderLeaders(full, legal, hooks);
  renderEvo(full, legal, hooks);
  renderCrests(full);
  renderMulligan(phase, acting, hooks);
  renderEndTurn(full, legal, phase, hooks);
  renderBonus(full, legal, hooks);
  renderHistory(s);
  renderChoice(full, legal, hooks);
  renderTerminal(full, hooks);
  renderEventLog(s);
  syncUndoButtons(s);
}

function renderHand(
  s: Session,
  full: FullState,
  legal: NeutralAction[],
  player: PlayerId,
  hide: boolean,
  hooks: RenderHooks,
): void {
  const id = `${visual(player)}Hand`;
  const el = byId(id);
  if (!el) return;
  const hand = full.players[player].hand;
  const phase = s.game.phase();
  const extras = hand
    .map((_, i) => {
      const playable = !!L.playAt(legal, player, i);
      const fuse = !!L.fuseAt(legal, player, i);
      return `${playable ? "p" : ""}${fuse ? "f" : ""}${s.mulliganSwap[i] ? "s" : ""}`;
    })
    .join(",");
  const sig = `${hide}|${phase}|${hand.map((c, i) => cardSignature(c, extras.split(",")[i] ?? "")).join(";")}`;
  if (zoneSig.get(id) === sig) return;
  zoneSig.set(id, sig);
  el.innerHTML = "";
  hand.forEach((inst, i) => {
    const playable = !!L.playAt(legal, player, i);
    const fuse = !!L.fuseAt(legal, player, i);
    const card = renderCard({
      inst,
      elementId: `${visual(player)}-hand-${i}`,
      faceDown: hide,
      glow: playable ? "legal-play" : undefined,
      selected: phase === "mulligan" && s.mulliganSwap[i],
      selectable: phase === "mulligan" || playable,
    });
    card.dataset.handPos = String(i);
    card.dataset.player = player;
    if (fuse) card.classList.add("fuse-ready");
    if (!hide) {
      if (phase === "mulligan") {
        card.addEventListener("click", (e) => {
          e.stopPropagation();
          hooks.onMulliganToggle(i);
        });
      } else {
        if (playable) {
          card.addEventListener("dblclick", (e) => {
            e.stopPropagation();
            hooks.onPlay(player, i);
          });
        }
        if (fuse) {
          card.addEventListener("contextmenu", (e) => {
            e.preventDefault();
            hooks.onFuse(player, i);
          });
        }
      }
    }
    el.appendChild(card);
  });
}

function renderBoard(
  full: FullState,
  legal: NeutralAction[],
  player: PlayerId,
  hooks: RenderHooks,
): void {
  const id = `${visual(player)}Board`;
  const el = byId(id);
  if (!el) return;
  const field = full.players[player].field;
  const sig = field
    .map((c, i) => {
      const atk = L.attacksFrom(legal, player, i).length;
      const ev = L.evolveFor(legal, player, false).some((a) => "evolve" in a && a.evolve.slot === i);
      const sev = L.evolveFor(legal, player, true).some((a) => "evolve" in a && a.evolve.slot === i);
      const en = !!L.engageAt(legal, player, i);
      return cardSignature(c, `${atk}:${ev}:${sev}:${en}`);
    })
    .join(";");
  if (zoneSig.get(id) === sig) {
    // still need live handlers if legal changed — rebuild when sig includes legal
    return;
  }
  zoneSig.set(id, sig);
  el.innerHTML = "";
  field.forEach((inst, i) => {
    if (!inst) {
      const empty = document.createElement("div");
      empty.className = "card empty-slot";
      empty.dataset.slot = String(i);
      empty.dataset.player = player;
      el.appendChild(empty);
      return;
    }
    const attacks = L.attacksFrom(legal, player, i);
    const engage = L.engageAt(legal, player, i);
    const card = renderCard({
      inst,
      elementId: `${visual(player)}-board-${i}`,
      onBoard: true,
      glow: attacks.length ? "can-attack legal-attack" : undefined,
      selectable: !!engage,
    });
    card.dataset.slot = String(i);
    card.dataset.player = player;
    if (engage) {
      card.classList.add("engage-ready");
      card.addEventListener("click", (e) => {
        e.stopPropagation();
        hooks.onEngage(player, i);
      });
    }
    if (attacks.length) card.classList.add("can-attack");
    if (inst.traits?.includes("storm") || inst.traits?.includes("rush")) {
      if (attacks.length) card.classList.add("rush-glow");
    }
    el.appendChild(card);
  });
}

function renderLeaders(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const el = byId(`${visual(p)}Leader`);
    if (!el) continue;
    el.classList.remove("selectable", "legal-target");
    const enemy = p === "a" ? "b" : "a";
    const hit = legal.some(
      (a) =>
        "attack" in a &&
        a.attack.player === enemy &&
        a.attack.target === "leader",
    );
    if (hit) {
      el.classList.add("legal-target");
      el.onclick = (e) => {
        e.stopPropagation();
        const atk = legal.find(
          (a) =>
            "attack" in a &&
            a.attack.player === enemy &&
            a.attack.target === "leader",
        );
        if (atk && "attack" in atk) {
          hooks.onAttack(atk.attack.player, atk.attack.attacker_slot, "leader");
        }
      };
    } else {
      el.onclick = null;
    }
    const cap = full.players[p].leader_mods?.some((m) => m.damage_cap != null);
    el.classList.toggle("has-barrier", !!cap);
  }
}

function renderEvo(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  const map: Array<{ id: string; player: PlayerId; superEvo: boolean }> = [
    { id: "blueNormalEvo", player: "a", superEvo: false },
    { id: "blueSuperEvo", player: "a", superEvo: true },
    { id: "redNormalEvo", player: "b", superEvo: false },
    { id: "redSuperEvo", player: "b", superEvo: true },
  ];
  for (const row of map) {
    const btn = byId<HTMLButtonElement>(row.id);
    if (!btn) continue;
    const acts = L.evolveFor(legal, row.player, row.superEvo);
    const p = full.players[row.player];
    btn.textContent = row.superEvo ? `Super (${p.sep})` : `Evo (${p.ep})`;
    btn.disabled = acts.length === 0;
    btn.onclick = (e) => {
      e.stopPropagation();
      if (!acts.length) return;
      if (acts.length === 1 && "evolve" in acts[0]) {
        hooks.onEvolve(row.player, acts[0].evolve.slot, row.superEvo);
      } else {
        hooks.evoArmed = { player: row.player, superEvo: row.superEvo };
        highlightEvolveSlots(legal, row.player, row.superEvo);
      }
    };
  }
}

function highlightEvolveSlots(
  legal: NeutralAction[],
  player: PlayerId,
  superEvo: boolean,
): void {
  for (const a of L.evolveFor(legal, player, superEvo)) {
    if (!("evolve" in a)) continue;
    const el = byId(`${visual(player)}-board-${a.evolve.slot}`);
    el?.classList.add("selectable", "legal-target");
  }
}

function renderCrests(full: FullState): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const box = byId(`${visual(p)}Crests`);
    if (!box) continue;
    const slots = Array.from(box.querySelectorAll<HTMLElement>(".crest-slot"));
    const crests = full.players[p].crests || [];
    slots.forEach((slot, i) => renderCrestSlot(slot, crests[i]));
    attachCrestTooltips(slots, crests, p);
  }
}

function attachCrestTooltips(
  slots: HTMLElement[],
  crests: FullState["players"]["a"]["crests"],
  player: PlayerId,
): void {
  const tip = byId("cardTooltip");
  if (!tip) return;
  slots.forEach((slot, i) => {
    const crest = crests[i];
    if (!crest) return;
    const info = lookupText(crest.id);
    slot.onmouseenter = () => {
      tip.textContent = `${info.name}\n${info.text}`;
      tip.style.display = "block";
    };
    slot.onmousemove = (e) => {
      const offsetY = player === "a" ? -tip.offsetHeight - 12 : 12;
      tip.style.left = `${Math.min(e.clientX + 12, window.innerWidth - tip.offsetWidth - 12)}px`;
      tip.style.top = `${Math.max(e.clientY + offsetY, 12)}px`;
    };
    slot.onmouseleave = () => {
      tip.style.display = "none";
    };
  });
}

function renderMulligan(
  phase: string,
  acting: PlayerId,
  hooks: RenderHooks,
): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const btn = byId<HTMLButtonElement>(`${visual(p)}MulliganConfirm`);
    if (!btn) continue;
    const show = phase === "mulligan" && acting === p;
    btn.style.display = show ? "inline-block" : "none";
    btn.onclick = () => hooks.onMulliganConfirm(p);
  }
}

function renderEndTurn(
  full: FullState,
  legal: NeutralAction[],
  phase: string,
  hooks: RenderHooks,
): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const btn = byId<HTMLButtonElement>(`endTurn${p === "a" ? "Blue" : "Red"}`);
    if (!btn) continue;
    const act = L.endTurn(legal, p);
    const show = phase !== "mulligan" && phase !== "terminal" && full.active === p;
    btn.style.display = show ? "inline-block" : "none";
    btn.disabled = !act;
    btn.onclick = () => {
      if (act) hooks.onEndTurn(p);
    };
  }
}

function renderBonus(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  const btn = byId<HTMLButtonElement>("redBoost");
  const early = byId("boostPipEarly");
  const late = byId("boostPipLate");
  const second: PlayerId = full.players.a.is_second ? "a" : "b";
  const bp = full.players[second].bonus_pp;
  early?.classList.toggle("used", bp.early_charge <= 0);
  late?.classList.toggle("used", bp.late_charge <= 0);
  const earlyTier = full.players[second].turns_taken < 5 || full.turn <= 5;
  early?.classList.toggle("active-tier", earlyTier && bp.early_charge > 0);
  late?.classList.toggle("active-tier", !earlyTier && bp.late_charge > 0);
  if (!btn) return;
  const act = L.bonusPp(legal, second);
  btn.disabled = !act;
  btn.classList.toggle("used", bp.active);
  btn.classList.toggle("disabled", !act);
  btn.onclick = () => {
    if (act) hooks.onBonusPp(second);
  };
}

function renderHistory(s: Session): void {
  fillHist("bluePlayedList", s.played.a);
  fillHist("redPlayedList", s.played.b);
  fillHist("blueDestroyedList", s.destroyed.a);
  fillHist("redDestroyedList", s.destroyed.b);
}

function fillHist(id: string, ids: string[]): void {
  const el = byId(id);
  if (!el) return;
  const sig = ids.join(",");
  if (zoneSig.get(id) === sig) return;
  zoneSig.set(id, sig);
  el.innerHTML = "";
  const ul = document.createElement("ul");
  ul.className = "hist-list";
  for (const card of ids) {
    const li = document.createElement("li");
    li.className = "hist-item";
    li.dataset.card = card;
    const name = lookupText(card).name;
    li.innerHTML = `<span class="hist-label">${escapeHtml(name)}</span>`;
    ul.appendChild(li);
  }
  el.appendChild(ul);
}

function renderChoice(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  document.querySelector(".choice-modal")?.remove();
  const confirmHost = byId("targetingConfirmation");
  if (confirmHost) confirmHost.style.display = "none";
  const phase = full.phase;
  if (typeof phase !== "object" || !("choice" in phase)) return;
  const node = phase.choice.node;
  const confirmAct = L.confirm(legal);
  if (confirmAct && confirmHost) {
    confirmHost.style.display = "block";
    confirmHost.innerHTML = `<button type="button" class="confirm-targets-btn">Confirm</button>`;
    confirmHost.querySelector("button")?.addEventListener("click", () => hooks.onConfirm());
  }
  if (isBoardChoice(node)) {
    highlightChoiceTargets(node, legal, hooks);
    return;
  }
  const modal = document.createElement("div");
  modal.className = "choice-modal";
  const opts = choiceLabels(node);
  modal.innerHTML = `<div class="choice-modal-content"><h3>Choose</h3><div class="choice-options">${opts
    .map(
      (o, i) =>
        `<button type="button" class="choice-option" data-index="${i}">${escapeHtml(o)}</button>`,
    )
    .join("")}</div></div>`;
  modal.querySelectorAll<HTMLButtonElement>(".choice-option").forEach((btn) => {
    btn.addEventListener("click", () => {
      const i = Number(btn.dataset.index);
      const act = chooseByIndex(legal, node, i);
      if (act) hooks.onChooseOpt(act);
    });
  });
  document.body.appendChild(modal);
}

function isBoardChoice(node: ChoiceNode): boolean {
  if ("targets" in node) {
    return node.targets.options.some((o) => "slot" in o || "leader" in o);
  }
  if ("multi_pick" in node) {
    return node.multi_pick.options.some((o) => "slot" in o || "leader" in o);
  }
  return false;
}

function highlightChoiceTargets(
  node: ChoiceNode,
  legal: NeutralAction[],
  hooks: RenderHooks,
): void {
  const options =
    "targets" in node
      ? node.targets.options
      : "multi_pick" in node
        ? node.multi_pick.options
        : [];
  options.forEach((opt) => {
    const el = targetEl(opt);
    if (!el) return;
    el.classList.add("selectable", "legal-target");
    el.addEventListener(
      "click",
      (e) => {
        e.stopPropagation();
        const act = chooseForTarget(legal, opt);
        if (act) hooks.onChooseOpt(act);
      },
      { once: true },
    );
  });
}

function targetEl(opt: TargetOpt): HTMLElement | null {
  if ("slot" in opt && "player" in opt && typeof opt.slot === "number") {
    return byId(`${visual(opt.player)}-board-${opt.slot}`);
  }
  if ("leader" in opt) return byId(`${visual(opt.leader)}Leader`);
  if ("hand" in opt) {
    return byId(`${visual(opt.hand.player)}-hand-${opt.hand.pos}`);
  }
  return null;
}

function chooseForTarget(legal: NeutralAction[], opt: TargetOpt): NeutralAction | null {
  return (
    legal.find((a) => {
      if (!("choose" in a)) return false;
      const o = a.choose.option;
      if (opt && "slot" in opt && typeof o === "object" && o && "slot" in o) {
        const sameSlot = o.slot === opt.slot;
        const samePlayer = !("player" in o) || !o.player || o.player === opt.player;
        return sameSlot && samePlayer;
      }
      if ("leader" in opt && o === "leader") return true;
      if ("card" in opt && typeof o === "object" && o && "card" in o) return o.card === opt.card;
      return false;
    }) ?? null
  );
}

function choiceLabels(node: ChoiceNode): string[] {
  if ("modes" in node) return node.modes.options.map((m) => `Mode ${m}`);
  if ("cards" in node) return node.cards.options.map((id) => lookupText(id).name);
  if ("fuse_partners" in node) {
    return node.fuse_partners.options.map((pos) => `Partner #${pos + 1}`);
  }
  if ("targets" in node) return node.targets.options.map(labelOpt);
  if ("multi_pick" in node) return node.multi_pick.options.map(labelOpt);
  return [];
}

function labelOpt(opt: TargetOpt): string {
  if ("card" in opt) return lookupText(opt.card).name;
  if ("mode" in opt) return `Mode ${opt.mode}`;
  if ("leader" in opt) return `Leader ${opt.leader.toUpperCase()}`;
  if ("slot" in opt) return `Slot ${opt.slot} (${opt.player})`;
  if ("hand" in opt) return `Hand ${opt.hand.pos}`;
  return "Option";
}

function chooseByIndex(
  legal: NeutralAction[],
  node: ChoiceNode,
  index: number,
): NeutralAction | null {
  if ("cards" in node) {
    const id = node.cards.options[index];
    return legal.find((a) => "choose" in a && typeof a.choose.option === "object" && a.choose.option && "card" in a.choose.option && a.choose.option.card === id) ?? null;
  }
  if ("modes" in node) {
    const mode = node.modes.options[index];
    return (
      legal.find(
        (a) =>
          "choose" in a &&
          typeof a.choose.option === "object" &&
          a.choose.option &&
          "mode" in a.choose.option &&
          a.choose.option.mode === mode,
      ) ?? legal.filter((a) => "choose" in a)[index] ?? null
    );
  }
  if ("fuse_partners" in node) {
    return legal.filter((a) => "choose" in a)[index] ?? null;
  }
  return legal.filter((a) => "choose" in a)[index] ?? null;
}

function renderTerminal(full: FullState, hooks: RenderHooks): void {
  let overlay = document.getElementById("gameOverOverlay");
  if (full.winner == null && (typeof full.phase === "string" ? full.phase !== "terminal" : true)) {
    if (overlay) overlay.style.display = "none";
    return;
  }
  if (typeof full.phase === "string" && full.phase !== "terminal") {
    if (overlay) overlay.style.display = "none";
    return;
  }
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.id = "gameOverOverlay";
    overlay.innerHTML = `<div class="gameover-card">
      <div class="gameover-title" id="gameOverTitle"></div>
      <div class="gameover-reason" id="gameOverReason">Match over</div>
      <div class="gameover-actions">
        <button type="button" id="newGameFromOver">New Game</button>
        <button type="button" id="rematchSwapBtn">Rematch (swap sides)</button>
      </div>
    </div>`;
    document.body.appendChild(overlay);
    overlay.querySelector("#newGameFromOver")?.addEventListener("click", () => hooks.onNewGame());
    overlay.querySelector("#rematchSwapBtn")?.addEventListener("click", () => hooks.onRematchSwap());
  }
  const title = document.getElementById("gameOverTitle");
  if (title) {
    title.textContent =
      full.winner === "a" ? "Blue (A) wins" : full.winner === "b" ? "Red (B) wins" : "Draw";
  }
  overlay.style.display = "flex";
}

function renderEventLog(s: Session): void {
  const el = byId("eventLog");
  if (!el) return;
  el.textContent = s.events.map((e) => JSON.stringify(e)).join("\n");
  el.dataset.count = String(s.events.length);
}

function syncUndoButtons(s: Session): void {
  const undo = byId<HTMLButtonElement>("undoBtn");
  const redo = byId<HTMLButtonElement>("redoBtn");
  if (undo) undo.disabled = s.past.length === 0;
  if (redo) redo.disabled = s.future.length === 0;
}

export function resetZoneCache(): void {
  zoneSig.clear();
}

export function bindTooltips(): void {
  const tip = byId("cardTooltip");
  const preview = byId("cardPreview");
  if (!tip) return;
  document.addEventListener("mouseover", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLElement>(".card[data-card]");
    if (!card || card.dataset.faceDown === "1") return;
    const id = card.dataset.card;
    if (!id) return;
    const info = lookupText(id);
    tip.innerHTML = `<strong>${escapeHtml(info.name)}</strong>\n${escapeHtml(info.text)}`;
    tip.style.display = "block";
    if (preview) {
      const url = cardImageUrl(id, card.classList.contains("evolved") || card.classList.contains("super-evo"));
      preview.innerHTML = "";
      if (url) {
        const img = document.createElement("img");
        img.referrerPolicy = "no-referrer";
        img.src = url;
        preview.appendChild(img);
        preview.style.display = "block";
      }
    }
  });
  document.addEventListener("mousemove", (e) => {
    if (tip.style.display === "none") return;
    tip.style.left = `${Math.min(e.clientX + 16, window.innerWidth - tip.offsetWidth - 12)}px`;
    tip.style.top = `${Math.min(e.clientY + 16, window.innerHeight - tip.offsetHeight - 12)}px`;
    if (preview) {
      preview.style.left = `${Math.max(12, e.clientX - 240)}px`;
      preview.style.top = `${Math.max(12, e.clientY - 20)}px`;
    }
  });
  document.addEventListener("mouseout", (e) => {
    const card = (e.target as HTMLElement).closest(".card[data-card]");
    if (!card) return;
    tip.style.display = "none";
    if (preview) preview.style.display = "none";
  });
}
