import { lookupText } from "./catalog.ts";
import { escapeHtml } from "./images.ts";
import { badgeCost, sessionBoardInfo, sessionHandInfo, usablePp } from "./info.ts";
import * as L from "./legal.ts";
import type { Session } from "./session.ts";
import { fullState, legalActions } from "./session.ts";
import type {
  BoardCardInfo,
  ChoiceNode,
  FullState,
  GateInfo,
  HandCardInfo,
  NeutralAction,
  PlayerId,
  TargetOpt,
} from "./types.ts";
import { attachPointerDragSource, setDropTarget } from "./drag.ts";
import { glowFor, renderCard, renderCrestSlot } from "./render/card.ts";
import { byId, setText, visual } from "./render/ids.ts";
import { formatCardTooltip, formatCrestTooltip } from "./tooltip.ts";

export type Pending =
  | { kind: "attack"; player: PlayerId; slot: number }
  | { kind: "evolve"; player: PlayerId; superEvo: boolean }
  | { kind: "play"; player: PlayerId; handPos: number }
  | null;

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
  pending: Pending;
  setPending: (p: Pending) => void;
};

const zoneSig = new Map<string, string>();
const gateByUid = new Map<number, GateInfo[]>();
const handInfoByUid = new Map<number, HandCardInfo>();

export function render(s: Session, hooks: RenderHooks): void {
  const t0 = performance.now();
  const full = fullState(s);
  const legal = legalActions(s);
  const acting = s.game.acting() as PlayerId;
  const phase = s.game.phase();
  const handA = sessionHandInfo(s, "a");
  const handB = sessionHandInfo(s, "b");
  const boardA = sessionBoardInfo(s, "a");
  const boardB = sessionBoardInfo(s, "b");
  cacheInfo(full, handA, handB, boardA, boardB);

  document.body.classList.toggle("active-first", full.active === "a");
  document.body.classList.toggle("active-second", full.active === "b");
  document.body.classList.toggle("gameover", phase === "terminal");
  document.body.classList.toggle("select-mode", phase === "choice" || hooks.pending != null);
  document.body.classList.toggle("mode-watch", s.cfg.mode === "watch");
  document.body.classList.toggle("mode-vs-bot", s.cfg.mode === "vs-bot");
  document.body.classList.toggle("mode-hotseat", s.cfg.mode === "hotseat");
  document.body.classList.toggle("awaiting-target", hooks.pending != null);

  const aUse = usablePp(full.players.a.pp, full.players.a.bonus_pp.active);
  const bUse = usablePp(full.players.b.pp, full.players.b.bonus_pp.active);
  setText("blueHP", full.players.a.leader_defense);
  setText("redHP", full.players.b.leader_defense);
  setText("bluePP", `${aUse}/${full.players.a.pp_max}`);
  setText("redPP", `${bUse}/${full.players.b.pp_max}`);
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

  renderHand(s, full, legal, "a", hideA, handA, hooks);
  renderHand(s, full, legal, "b", hideB, handB, hooks);
  renderBoard(full, legal, "a", boardA, hooks);
  renderBoard(full, legal, "b", boardB, hooks);
  bindBoardDrops(hooks);
  renderLeaders(full, legal, hooks);
  renderEvo(full, legal, hooks);
  renderCrests(full);
  renderMulligan(phase, acting, hooks);
  renderEndTurn(full, legal, phase, hooks);
  renderBonus(full, legal);
  renderHistory(s);
  renderChoice(full, legal, hooks);
  renderTerminal(full, hooks);
  renderEventLog(s);
  syncUndoButtons(s);
  paintPending(hooks.pending, legal);
  if (window.__arena) window.__arena.paintMs = performance.now() - t0;
}

function cacheInfo(
  full: FullState,
  handA: HandCardInfo[],
  handB: HandCardInfo[],
  boardA: BoardCardInfo[],
  boardB: BoardCardInfo[],
): void {
  gateByUid.clear();
  handInfoByUid.clear();
  const putHand = (list: HandCardInfo[], player: PlayerId) => {
    const hand = full.players[player].hand;
    list.forEach((info, i) => {
      const inst = hand[i];
      if (!inst) return;
      handInfoByUid.set(inst.id, info);
      gateByUid.set(inst.id, info.gates);
    });
  };
  const putBoard = (list: BoardCardInfo[], player: PlayerId) => {
    const field = full.players[player].field;
    for (const info of list) {
      const inst = field[info.slot];
      if (!inst) continue;
      gateByUid.set(inst.id, info.gates);
    }
  };
  putHand(handA, "a");
  putHand(handB, "b");
  putBoard(boardA, "a");
  putBoard(boardB, "b");
}

function renderHand(
  s: Session,
  full: FullState,
  legal: NeutralAction[],
  player: PlayerId,
  hide: boolean,
  infos: HandCardInfo[],
  hooks: RenderHooks,
): void {
  const id = `${visual(player)}Hand`;
  const el = byId(id);
  if (!el) return;
  const hand = full.players[player].hand;
  const phase = s.game.phase();
  const byPos = new Map(infos.map((i) => [i.pos, i]));
  syncKeyed(
    el,
    hand.map((inst, i) => ({ key: String(inst.id), inst, i })),
    (row) => {
      const info = byPos.get(row.i);
      const playable = !!info?.playable || !!L.playAt(legal, player, row.i);
      const fuse = !!L.fuseAt(legal, player, row.i);
      const gateMet = !!info?.gates.some((g) => g.met);
      const pendingPlay =
        hooks.pending?.kind === "play" &&
        hooks.pending.player === player &&
        hooks.pending.handPos === row.i;
      const card = renderCard({
        inst: row.inst,
        elementId: `${visual(player)}-hand-${row.inst.id}`,
        faceDown: hide,
        glow: hide ? undefined : glowFor({ playable, gateMet, form: info?.form }),
        selected: (phase === "mulligan" && s.mulliganSwap[row.i]) || pendingPlay,
        selectable: !hide && phase === "mulligan",
        displayCost: hide ? undefined : badgeCost(info, row.inst.cost),
        form: hide ? undefined : info?.form,
      });
      card.dataset.handPos = String(row.i);
      card.dataset.player = player;
      card.classList.toggle("fuse-ready", fuse);
      if (info) {
        handInfoByUid.set(row.inst.id, info);
        gateByUid.set(row.inst.id, info.gates);
      }
      if (!hide) {
        attachPointerDragSource(
          card,
          {
            payload: JSON.stringify({ kind: "play", player, handPos: row.i }),
            kind: "hand",
          },
          playable && phase !== "mulligan",
        );
      }
      return card;
    },
  );
}

function renderBoard(
  full: FullState,
  legal: NeutralAction[],
  player: PlayerId,
  infos: BoardCardInfo[],
  hooks: RenderHooks,
): void {
  const id = `${visual(player)}Board`;
  const el = byId(id);
  if (!el) return;
  el.classList.add("board-zone");
  const occupied = full.players[player].field
    .map((inst, i) => (inst ? { inst, i } : null))
    .filter((x): x is { inst: NonNullable<typeof x>["inst"]; i: number } => !!x);
  const bySlot = new Map(infos.map((i) => [i.slot, i]));
  syncKeyed(
    el,
    occupied.map((row) => ({ key: String(row.inst.id), ...row })),
    (row) => {
      const info = bySlot.get(row.i);
      const attacks = L.attacksFrom(legal, player, row.i);
      const engage = L.engageAt(legal, player, row.i);
      const canAttack = !!info?.can_attack || attacks.length > 0;
      const gateMet = !!info?.gates.some((g) => g.met);
      const pendingAtk =
        hooks.pending?.kind === "attack" &&
        hooks.pending.player === player &&
        hooks.pending.slot === row.i;
      const card = renderCard({
        inst: row.inst,
        elementId: `${visual(player)}-board-${row.inst.id}`,
        onBoard: true,
        glow: glowFor({ canAttack, gateMet }),
        selected: pendingAtk,
        selectable: !!engage || canAttack,
      });
      card.dataset.slot = String(row.i);
      card.dataset.player = player;
      if (info) gateByUid.set(row.inst.id, info.gates);
      if (engage) card.classList.add("engage-ready");
      if (row.inst.traits?.includes("storm") || row.inst.traits?.includes("rush")) {
        if (canAttack) card.classList.add("rush-glow");
      }
      attachPointerDragSource(
        card,
        {
          payload: JSON.stringify({ kind: "attack", player, slot: row.i }),
          kind: "attacker",
        },
        canAttack,
      );
      return card;
    },
  );
}

function bindBoardDrops(hooks: RenderHooks): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const board = byId(`${visual(p)}Board`);
    if (!board) continue;
    setDropTarget(
      board,
      (payload) => {
        const data = parsePayload(payload);
        return data?.kind === "play" && data.player === p;
      },
      (payload) => {
        const data = parsePayload(payload);
        if (data?.kind === "play") hooks.onPlay(data.player, data.handPos);
      },
    );
    const leader = byId(`${visual(p)}Leader`);
    if (leader) {
      setDropTarget(
        leader,
        (payload) => {
          const data = parsePayload(payload);
          return data?.kind === "attack" && data.player !== p;
        },
        (payload) => {
          const data = parsePayload(payload);
          if (data?.kind === "attack") hooks.onAttack(data.player, data.slot, "leader");
        },
      );
    }
  }
  document.querySelectorAll<HTMLElement>(".board-zone .card[data-slot]").forEach((card) => {
    const slot = Number(card.dataset.slot);
    const player = card.dataset.player as PlayerId;
    setDropTarget(
      card,
      (payload) => {
        const data = parsePayload(payload);
        return (
          (data?.kind === "attack" && data.player !== player) ||
          (data?.kind === "evo" && data.player === player)
        );
      },
      (payload) => {
        const data = parsePayload(payload);
        if (data?.kind === "attack") hooks.onAttack(data.player, data.slot, { slot });
        if (data?.kind === "evo") hooks.onEvolve(data.player, slot, data.superEvo);
      },
    );
  });
}

function parsePayload(payload: string): {
  kind: string;
  player: PlayerId;
  handPos: number;
  slot: number;
  superEvo: boolean;
} | null {
  try {
    return JSON.parse(payload);
  } catch {
    return null;
  }
}

function syncKeyed<T extends { key: string }>(
  host: HTMLElement,
  rows: T[],
  renderOne: (row: T) => HTMLElement,
): void {
  const prev = new Map<string, HTMLElement>();
  for (const child of Array.from(host.children) as HTMLElement[]) {
    const key = child.dataset.uid ?? child.id;
    prev.set(key, child);
  }
  const keep = new Set(rows.map((r) => r.key));
  for (const [key, node] of prev) {
    if (!keep.has(key)) {
      node.classList.add("card-leave");
      node.remove();
    }
  }
  const frag: HTMLElement[] = [];
  for (const row of rows) {
    const existed = prev.has(row.key);
    const node = renderOne(row);
    if (!existed) node.classList.add("card-enter");
    frag.push(node);
  }
  let changed = frag.length !== host.childElementCount;
  if (!changed) {
    for (let i = 0; i < frag.length; i++) {
      if (host.children[i] !== frag[i]) {
        changed = true;
        break;
      }
    }
  }
  if (!changed) return;
  for (const node of frag) host.appendChild(node);
}

function renderLeaders(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const el = byId(`${visual(p)}Leader`);
    if (!el) continue;
    el.classList.remove("selectable", "legal-target");
    const enemy = p === "a" ? "b" : "a";
    const hit = legal.some(
      (a) => "attack" in a && a.attack.player === enemy && a.attack.target === "leader",
    );
    const atk = hooks.pending?.kind === "attack" ? hooks.pending : null;
    const pending =
      !!atk &&
      atk.player === enemy &&
      legal.some(
        (a) =>
          "attack" in a &&
          a.attack.player === enemy &&
          a.attack.attacker_slot === atk.slot &&
          a.attack.target === "leader",
      );
    if (hit || pending) {
      el.classList.add("legal-target");
      if (pending) el.classList.add("selectable");
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
    btn.classList.toggle(
      "selected",
      hooks.pending?.kind === "evolve" &&
        hooks.pending.player === row.player &&
        hooks.pending.superEvo === row.superEvo,
    );
    attachPointerDragSource(
      btn,
      {
        payload: JSON.stringify({
          kind: "evo",
          player: row.player,
          superEvo: row.superEvo,
        }),
        kind: "evo",
      },
      acts.length > 0,
    );
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
    slot.onmouseenter = () => {
      tip.innerHTML = formatCrestTooltip(crest);
      tip.style.display = "block";
    };
    slot.onpointermove = (e) => {
      const offsetY = player === "a" ? -tip.offsetHeight - 12 : 12;
      tip.style.left = `${Math.min(e.clientX + 12, window.innerWidth - tip.offsetWidth - 12)}px`;
      tip.style.top = `${Math.max(e.clientY + offsetY, 12)}px`;
    };
    slot.onmouseleave = () => {
      tip.style.display = "none";
    };
  });
}

function renderMulligan(phase: string, acting: PlayerId, hooks: RenderHooks): void {
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

function renderBonus(full: FullState, legal: NeutralAction[]): void {
  const controls = byId("boostControls");
  const early = byId("boostPipEarly");
  const late = byId("boostPipLate");
  const btn = byId<HTMLButtonElement>("bonusPpBtn");
  const second: PlayerId = full.players.a.is_second ? "a" : "b";
  const host = byId(`${visual(second)}BoostHost`);
  if (controls && host && controls.parentElement !== host) host.appendChild(controls);
  if (controls) controls.hidden = false;
  const other = second === "a" ? "b" : "a";
  const otherHost = byId(`${visual(other)}BoostHost`);
  if (otherHost) otherHost.replaceChildren();
  const bp = full.players[second].bonus_pp;
  early?.classList.toggle("used", !bp.early_charge);
  late?.classList.toggle("used", !bp.late_charge);
  const earlyTier = full.players[second].turns_taken < 5 || full.turn <= 5;
  early?.classList.toggle("active-tier", earlyTier && bp.early_charge);
  late?.classList.toggle("active-tier", !earlyTier && bp.late_charge);
  if (!btn) return;
  const act = L.bonusPp(legal, second);
  btn.disabled = !act;
  btn.classList.toggle("used", bp.active);
  btn.classList.toggle("disabled", !act);
  btn.dataset.player = second;
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
    li.innerHTML = `<span class="hist-label">${escapeHtml(lookupText(card).name)}</span>`;
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
    .map((o, i) => `<button type="button" class="choice-option" data-index="${i}">${o}</button>`)
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
  if ("targets" in node) return node.targets.options.some((o) => "slot" in o || "leader" in o);
  if ("multi_pick" in node) return node.multi_pick.options.some((o) => "slot" in o || "leader" in o);
  return false;
}

function highlightChoiceTargets(
  node: ChoiceNode,
  legal: NeutralAction[],
  hooks: RenderHooks,
): void {
  const options =
    "targets" in node ? node.targets.options : "multi_pick" in node ? node.multi_pick.options : [];
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
    const board = byId(`${visual(opt.player)}Board`);
    return (
      board?.querySelector<HTMLElement>(`.card[data-slot="${opt.slot}"]`) ??
      byId(`${visual(opt.player)}-board-${opt.slot}`)
    );
  }
  if ("leader" in opt) return byId(`${visual(opt.leader)}Leader`);
  if ("hand" in opt) {
    return byId(`${visual(opt.hand.player)}Hand`)?.querySelector(
      `.card[data-hand-pos="${opt.hand.pos}"]`,
    ) as HTMLElement | null;
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
  if ("cards" in node) return node.cards.options.map((id) => escapeHtml(lookupText(id).name));
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
    return (
      legal.find(
        (a) =>
          "choose" in a &&
          typeof a.choose.option === "object" &&
          a.choose.option &&
          "card" in a.choose.option &&
          a.choose.option.card === id,
      ) ?? null
    );
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
      ) ??
      legal.filter((a) => "choose" in a)[index] ??
      null
    );
  }
  if ("fuse_partners" in node) return legal.filter((a) => "choose" in a)[index] ?? null;
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

function paintPending(pending: Pending, legal: NeutralAction[]): void {
  if (!pending) return;
  if (pending.kind === "attack") {
    for (const a of L.attacksFrom(legal, pending.player, pending.slot)) {
      if (!("attack" in a)) continue;
      if (a.attack.target === "leader") {
        const enemy = pending.player === "a" ? "b" : "a";
        byId(`${visual(enemy)}Leader`)?.classList.add("selectable", "legal-target");
      } else if (typeof a.attack.target === "object") {
        const enemy = pending.player === "a" ? "b" : "a";
        const el = byId(`${visual(enemy)}Board`)?.querySelector<HTMLElement>(
          `.card[data-slot="${a.attack.target.slot}"]`,
        );
        el?.classList.add("selectable", "legal-target");
      }
    }
  }
  if (pending.kind === "evolve") {
    for (const a of L.evolveFor(legal, pending.player, pending.superEvo)) {
      if (!("evolve" in a)) continue;
      const el = byId(`${visual(pending.player)}Board`)?.querySelector<HTMLElement>(
        `.card[data-slot="${a.evolve.slot}"]`,
      );
      el?.classList.add("selectable", "legal-target");
    }
  }
}

export function resetZoneCache(): void {
  zoneSig.clear();
}

export function bindTooltips(): void {
  const tip = byId("cardTooltip");
  if (!tip) return;
  document.addEventListener("mouseover", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLElement>(".card[data-card]");
    if (!card || card.dataset.faceDown === "1") return;
    const id = card.dataset.card;
    if (!id) return;
    const uid = Number(card.dataset.uid);
    const info = handInfoByUid.get(uid);
    tip.innerHTML = formatCardTooltip({
      cardId: id,
      gates: gateByUid.get(uid),
      displayCost: info?.cost ?? null,
    });
    tip.style.display = "block";
  });
  document.addEventListener("mousemove", (e) => {
    if (tip.style.display === "none") return;
    tip.style.left = `${Math.min(e.clientX + 16, window.innerWidth - tip.offsetWidth - 12)}px`;
    tip.style.top = `${Math.min(e.clientY + 16, window.innerHeight - tip.offsetHeight - 12)}px`;
  });
  document.addEventListener("mouseout", (e) => {
    const card = (e.target as HTMLElement).closest(".card[data-card]");
    if (!card) return;
    const next = (e.relatedTarget as HTMLElement | null)?.closest?.(".card[data-card]");
    if (next === card) return;
    tip.style.display = "none";
  });
}
