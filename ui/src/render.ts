import { getCatalog, lookupText } from "./catalog.ts";
import { cardImageUrl, escapeHtml } from "./images.ts";
import {
  badgeCost,
  conditionGateMet,
  sessionBoardInfo,
  sessionHandInfo,
  sessionPlayerInfo,
  usablePp,
} from "./info.ts";
import * as L from "./legal.ts";
import type { Session } from "./session.ts";
import { fullState, legalActions } from "./session.ts";
import type {
  BoardCardInfo,
  CardInstance,
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
import { releaseImageLoads } from "./releaseImages.ts";
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
  onRematchSame: () => void;
  onRematchNew: () => void;
  onUndo: () => void;
  pending: Pending;
  setPending: (p: Pending) => void;
};

const zoneSig = new Map<string, string>();
const gateByUid = new Map<number, GateInfo[]>();
const handInfoByUid = new Map<number, HandCardInfo>();

/** Current in-place choice — used to resolve slot owners when legal omits `player`. */
let currentChoiceNode: ChoiceNode | null = null;
let currentChoicePlayer: PlayerId | null = null;
let lastFull: FullState | null = null;
let tipEl: HTMLElement | null = null;
let tipSession: { card: HTMLElement; drag: boolean } | null = null;

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
  lastFull = full;
  cacheInfo(full, handA, handB, boardA, boardB);
  if (typeof full.phase !== "object" || !("choice" in full.phase)) {
    currentChoiceNode = null;
    currentChoicePlayer = null;
  }

  document.body.classList.toggle("active-first", full.active === "a");
  document.body.classList.toggle("active-second", full.active === "b");
  document.body.classList.toggle("gameover", phase === "terminal");
  document.body.classList.toggle("select-mode", phase === "choice");
  document.body.classList.toggle("pending-target", hooks.pending != null);
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
      ? full.winner === "a"
        ? "Blue (A) wins"
        : full.winner === "b"
          ? "Red (B) wins"
          : "Draw"
      : `Turn ${full.turn || 0} · ${acting.toUpperCase()}`,
  );

  const hideA = s.cfg.mode === "vs-bot" && s.cfg.hideBotHand && s.cfg.humanSide !== "a";
  const hideB = s.cfg.mode === "vs-bot" && s.cfg.hideBotHand && s.cfg.humanSide !== "b";

  renderHand(s, full, legal, "a", hideA, handA, hooks);
  renderHand(s, full, legal, "b", hideB, handB, hooks);
  renderBoard(full, legal, "a", boardA, hooks);
  renderBoard(full, legal, "b", boardB, hooks);
  bindBoardDrops(legal, hooks);
  renderLeaders(full, legal, hooks, s);
  renderEvo(s, full, legal, hooks);
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
  refreshOpenTooltip(full);
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
  const acting = s.game.acting() as PlayerId;
  syncKeyed(
    el,
    hand.map((inst, i) => ({ key: String(inst.id), inst, i })),
    (row) => {
      const info = byPos.get(row.i);
      const playable = !!L.playAt(legal, player, row.i);
      const fuse = !!L.fuseAt(legal, player, row.i);
      const yellow = conditionGateMet(info?.gates) || (!!info?.form && info.form !== "normal");
      const pendingPlay =
        hooks.pending?.kind === "play" &&
        hooks.pending.player === player &&
        hooks.pending.handPos === row.i;
      const mullSelected = phase === "mulligan" && acting === player && s.mulliganSwap[row.i];
      const card = renderCard({
        inst: row.inst,
        elementId: `${visual(player)}-hand-${row.inst.id}`,
        faceDown: hide,
        glow: hide ? undefined : glowFor({ playable, yellow, form: info?.form }),
        selected: mullSelected || pendingPlay,
        selectable: !hide && phase === "mulligan" && acting === player,
        displayCost: hide ? undefined : badgeCost(info, row.inst.cost),
        form: hide ? undefined : info?.form,
      });
      card.dataset.handPos = String(row.i);
      card.dataset.player = player;
      card.classList.toggle("fuse-ready", fuse);
      card.querySelector(".fuse-chip")?.remove();
      if (fuse && !hide) {
        const chip = document.createElement("button");
        chip.type = "button";
        chip.className = "fuse-chip";
        chip.textContent = "Fuse";
        chip.addEventListener("click", (ev) => {
          ev.stopPropagation();
          hooks.onFuse(player, row.i);
        });
        card.appendChild(chip);
      }
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
            handZoneId: id,
            onReleasedInHand: fuse ? () => hooks.onFuse(player, row.i) : undefined,
            onDragBegan: () => beginDragTooltip(card),
            onDragEnded: () => endDragTooltip(),
          },
          (playable || fuse) && phase !== "mulligan",
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
      const canAttack = attacks.length > 0;
      const hitsLeader = attacks.some((a) => "attack" in a && a.attack.target === "leader");
      // Yellow only with a legal follower-only attack on the entry turn.
      // `summoning_sick` stays true on the opponent's turn — do not paint from it alone.
      const rushOnly = canAttack && !hitsLeader && !!row.inst.flags?.summoning_sick;
      const pendingAtk =
        hooks.pending?.kind === "attack" &&
        hooks.pending.player === player &&
        hooks.pending.slot === row.i;
      const card = renderCard({
        inst: row.inst,
        elementId: `${visual(player)}-board-${row.inst.id}`,
        onBoard: true,
        glow: glowFor({ canAttack, rushOnly }),
        selected: pendingAtk,
        selectable: !!engage || canAttack,
        cannotAttack: !!info?.cannot_attack_reason,
      });
      card.dataset.slot = String(row.i);
      card.dataset.player = player;
      if (info) gateByUid.set(row.inst.id, info.gates);
      if (engage) card.classList.add("engage-ready");
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

function bindBoardDrops(legal: NeutralAction[], hooks: RenderHooks): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const board = byId(`${visual(p)}Board`);
    if (!board) continue;
    setDropTarget(
      board,
      (payload) => {
        const data = parsePayload(payload);
        return !!data && data.kind === "play" && !!L.playAt(legal, data.player, data.handPos);
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
          if (!data || data.kind !== "attack" || data.player === p) return false;
          return legal.some(
            (a) =>
              "attack" in a &&
              a.attack.player === data.player &&
              a.attack.attacker_slot === data.slot &&
              a.attack.target === "leader",
          );
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
        if (!data) return false;
        if (data.kind === "attack" && data.player !== player) {
          return legal.some(
            (a) =>
              "attack" in a &&
              a.attack.player === data.player &&
              a.attack.attacker_slot === data.slot &&
              typeof a.attack.target === "object" &&
              a.attack.target.slot === slot,
          );
        }
        if (data.kind === "evo" && data.player === player) {
          return legal.some(
            (a) =>
              "evolve" in a &&
              a.evolve.player === data.player &&
              a.evolve.slot === slot &&
              a.evolve.super === data.superEvo,
          );
        }
        return false;
      },
      (payload) => {
        const data = parsePayload(payload);
        if (data?.kind === "attack") hooks.onAttack(data.player, data.slot, { slot });
        if (data?.kind === "evo") hooks.onEvolve(data.player, slot, data.superEvo);
      },
    );
  });
}

function teardownKeyedNode(node: HTMLElement): void {
  if (node.classList.contains("spell")) {
    spawnSpellCast(node);
  } else {
    node.classList.add("card-leave");
  }
  releaseImageLoads(node);
  node.remove();
}

function spawnSpellCast(node: HTMLElement): void {
  const rect = node.getBoundingClientRect();
  const clone = node.cloneNode(true) as HTMLElement;
  clone.removeAttribute("id");
  clone.classList.remove("card-leave", "card-enter", "pressed");
  clone.classList.add("spell-cast");
  clone.style.position = "fixed";
  clone.style.left = `${rect.left}px`;
  clone.style.top = `${rect.top}px`;
  clone.style.width = `${rect.width}px`;
  clone.style.height = `${rect.height}px`;
  clone.style.margin = "0";
  clone.style.zIndex = "80";
  clone.style.pointerEvents = "none";
  document.body.appendChild(clone);
  window.setTimeout(() => {
    releaseImageLoads(clone);
    clone.remove();
  }, 500);
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
      teardownKeyedNode(node);
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

function renderLeaders(
  full: FullState,
  legal: NeutralAction[],
  hooks: RenderHooks,
  s: Session,
): void {
  for (const p of ["a", "b"] as PlayerId[]) {
    const el = byId(`${visual(p)}Leader`);
    if (!el) continue;
    el.classList.remove("selectable", "legal-target");
    const enemy = p === "a" ? "b" : "a";
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
    if (pending) {
      el.classList.add("legal-target", "selectable");
    }
    const info = sessionPlayerInfo(s, p);
    const cap =
      !!info.has_leader_barrier ||
      !!full.players[p].leader_mods?.some((m) => m.damage_cap != null);
    el.classList.toggle("has-barrier", cap);
    el.classList.toggle("has-leader-barrier", cap);
  }
}

function renderEvo(
  s: Session,
  full: FullState,
  legal: NeutralAction[],
  hooks: RenderHooks,
): void {
  const map: Array<{ id: string; player: PlayerId; superEvo: boolean }> = [
    { id: "blueNormalEvo", player: "a", superEvo: false },
    { id: "blueSuperEvo", player: "a", superEvo: true },
    { id: "redNormalEvo", player: "b", superEvo: false },
    { id: "redSuperEvo", player: "b", superEvo: true },
  ];
  const infoByPlayer: Record<PlayerId, ReturnType<typeof sessionPlayerInfo>> = {
    a: sessionPlayerInfo(s, "a"),
    b: sessionPlayerInfo(s, "b"),
  };
  for (const row of map) {
    const btn = byId<HTMLButtonElement>(row.id);
    if (!btn) continue;
    const acts = L.evolveFor(legal, row.player, row.superEvo);
    const p = full.players[row.player];
    const info = infoByPlayer[row.player];
    const unlocked = row.superEvo ? info.super_evolve_unlocked : info.evolve_unlocked;
    const remain = row.superEvo ? info.super_evolve_unlock_in : info.evolve_unlock_in;
    const base = row.superEvo ? `Super (${p.sep})` : `Evo (${p.ep})`;
    const label = document.createElement("span");
    label.className = "evo-btn-label";
    label.textContent = !unlocked && remain > 0 ? `${base} · ${remain}` : base;
    btn.replaceChildren(label);
    btn.disabled = !unlocked;
    btn.classList.toggle("evo-locked", !unlocked);
    if (!unlocked && remain > 0) {
      btn.title = remain === 1 ? "unlocks in 1 turn" : `unlocks in ${remain} turns`;
    } else {
      btn.removeAttribute("title");
    }
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
    const faith = full.players[p].faith ?? 0;
    slots.forEach((slot, i) => renderCrestSlot(slot, crests[i], faith));
    attachCrestTooltips(slots, crests, p, faith);
  }
}

function attachCrestTooltips(
  slots: HTMLElement[],
  crests: FullState["players"]["a"]["crests"],
  player: PlayerId,
  faith: number,
): void {
  const tip = byId("cardTooltip");
  if (!tip) return;
  slots.forEach((slot, i) => {
    const crest = crests[i];
    if (!crest) return;
    slot.onmouseenter = () => {
      tip.innerHTML = formatCrestTooltip(crest, faith);
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
    btn.classList.toggle("disabled", !act);
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
  const groups = new Map<
    string,
    { name: string; cost: number; count: number; card: string; set?: number | null }
  >();
  for (const card of ids) {
    const info = lookupText(card);
    const cat = getCatalog(card);
    const cost = info.cost ?? cat?.cost ?? 0;
    const name = info.name || card;
    const key = `${name}||${cost}`;
    const g = groups.get(key) || { name, cost, count: 0, card, set: info.set };
    g.count += 1;
    if (g.set == null && info.set != null) g.set = info.set;
    groups.set(key, g);
  }
  const sorted = [...groups.values()].sort(
    (a, b) => a.cost - b.cost || a.name.localeCompare(b.name),
  );
  const ul = document.createElement("ul");
  ul.className = "hist-list";
  for (const g of sorted) {
    const li = document.createElement("li");
    li.className = "hist-item";
    li.dataset.card = g.card;
    const art = cardImageUrl(g.card, false);
    if (art) li.dataset.img = art;
    const set =
      g.set != null
        ? `<span class="hist-set"${g.set < 8 ? ' data-older="1"' : ""}>Set ${g.set}</span>`
        : "";
    li.innerHTML =
      `<span class="cost-badge">${g.cost}</span>` +
      `<span class="hist-label">${escapeHtml(g.name)} ×${g.count} ${set}</span>`;
    ul.appendChild(li);
  }
  el.appendChild(ul);
  wireHistoryPreview();
}

let histPreviewWired = false;

function wireHistoryPreview(): void {
  if (histPreviewWired) return;
  histPreviewWired = true;
  const preview = document.createElement("div");
  preview.id = "historyImgPreview";
  document.body.appendChild(preview);
  document.addEventListener("mousemove", (e) => {
    if (preview.style.display !== "block") return;
    let x = e.clientX + 18;
    let y = e.clientY + 18;
    if (x + 210 > window.innerWidth) x = window.innerWidth - 210 - 12;
    if (y + 300 > window.innerHeight) y = window.innerHeight - 300 - 12;
    preview.style.left = `${Math.max(12, x)}px`;
    preview.style.top = `${Math.max(12, y)}px`;
  });
  document.addEventListener("mouseover", (e) => {
    const li = (e.target as HTMLElement).closest<HTMLElement>(".hist-item");
    if (!li?.dataset.img) return;
    preview.innerHTML = "";
    const img = document.createElement("img");
    img.width = 198;
    img.alt = "";
    img.src = li.dataset.img;
    img.referrerPolicy = "no-referrer";
    preview.appendChild(img);
    preview.style.display = "block";
  });
  document.addEventListener("mouseout", (e) => {
    const li = (e.target as HTMLElement).closest(".hist-item");
    if (!li) return;
    preview.innerHTML = "";
    preview.style.display = "none";
  });
}

let promptPlaceCleanup: (() => void) | null = null;

function renderChoice(full: FullState, legal: NeutralAction[], hooks: RenderHooks): void {
  document.querySelector(".choice-modal")?.remove();
  promptPlaceCleanup?.();
  document.querySelector(".choice-prompt-bar")?.remove();
  const confirmHost = byId("targetingConfirmation");
  if (confirmHost) {
    confirmHost.style.display = "none";
    confirmHost.replaceChildren();
  }
  const phase = full.phase;
  if (typeof phase !== "object" || !("choice" in phase)) return;
  const node = phase.choice.node;
  const acting = phase.choice.player;
  currentChoiceNode = node;
  currentChoicePlayer = acting;
  const confirmAct = L.confirm(legal);
  const chooses = legal.filter((a) => "choose" in a);
  const inPlace = isInPlaceChoice(node);
  paintChoicePrompt(choicePrompt(node), hooks, confirmAct ? () => hooks.onConfirm() : null, node);

  if (inPlace) {
    highlightChoiceTargets(legal, acting, node);
    return;
  }

  const modal = document.createElement("div");
  modal.className = "choice-modal";
  const title = "modes" in node ? "Choose an effect:" : choicePrompt(node);
  const buttons = chooses.map((act, i) => {
    const label = labelChooseAction(act, node, full, acting);
    const earth = /earth rite|sigil/i.test(label)
      ? `<span class="earth-rite-cost">(Consume Earth Sigil)</span>`
      : "";
    return `<button type="button" class="choice-option" data-index="${i}">${escapeHtml(label)}${earth}</button>`;
  });
  modal.innerHTML = `<div class="choice-modal-content"><h3>${escapeHtml(title)}</h3><div class="choice-options">${buttons.join("")}</div><div class="choice-modal-actions"></div></div>`;
  const actions = modal.querySelector(".choice-modal-actions");
  if (confirmAct && actions) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "confirm-targets-btn";
    btn.textContent = confirmLabel(node);
    btn.addEventListener("click", () => hooks.onConfirm());
    actions.appendChild(btn);
  }
  modal.querySelectorAll<HTMLButtonElement>(".choice-option").forEach((btn) => {
    btn.addEventListener("click", () => {
      btn.classList.add("processing");
      modal.remove();
      const i = Number(btn.dataset.index);
      const act = chooses[i];
      if (act) hooks.onChooseOpt(act);
    });
  });
  document.body.appendChild(modal);
}

function isInPlaceChoice(node: ChoiceNode): boolean {
  if ("fuse_partners" in node) return true;
  if ("targets" in node) {
    return node.targets.options.some((o) => "slot" in o || "leader" in o || "hand" in o);
  }
  if ("multi_pick" in node) {
    return node.multi_pick.options.some((o) => "slot" in o || "leader" in o || "hand" in o);
  }
  return false;
}

function highlightChoiceTargets(
  legal: NeutralAction[],
  acting: PlayerId,
  node: ChoiceNode,
): void {
  const mark = (el: HTMLElement | null) => {
    el?.classList.add("selectable", "legal-target");
  };
  if ("fuse_partners" in node) {
    const picked = new Set(node.fuse_partners.picked);
    for (const pos of node.fuse_partners.options) {
      if (picked.has(pos)) continue;
      mark(
        byId(`${visual(acting)}Hand`)?.querySelector<HTMLElement>(
          `.card[data-hand-pos="${pos}"]`,
        ) ?? null,
      );
    }
    return;
  }
  const opts = nodeTargetOptions(node);
  const picked = "multi_pick" in node ? new Set(node.multi_pick.picked) : new Set<number>();
  opts.forEach((o, i) => {
    if (picked.has(i)) return;
    if ("slot" in o) {
      mark(
        byId(`${visual(o.player)}Board`)?.querySelector<HTMLElement>(
          `.card[data-slot="${o.slot}"]`,
        ) ?? null,
      );
      return;
    }
    if ("leader" in o) {
      mark(byId(`${visual(o.leader)}Leader`));
      return;
    }
    if ("hand" in o && o.hand.player === acting) {
      mark(
        byId(`${visual(acting)}Hand`)?.querySelector<HTMLElement>(
          `.card[data-hand-pos="${o.hand.pos}"]`,
        ) ?? null,
      );
    }
  });
  void legal;
}

function paintPromptBar(opts: {
  text: string;
  undo?: () => void;
  confirm?: () => void;
  confirmLabel?: string;
  cancel?: () => void;
}): void {
  promptPlaceCleanup?.();
  document.querySelector(".choice-prompt-bar")?.remove();
  document.querySelector(".pending-cancel-chip")?.remove();
  const bar = document.createElement("div");
  bar.className = "choice-prompt-bar";
  bar.id = "promptBar";
  const label = document.createElement("span");
  label.className = "choice-prompt-text";
  label.textContent = opts.text;
  bar.appendChild(label);
  if (opts.undo) {
    const undo = document.createElement("button");
    undo.type = "button";
    undo.className = "undo-chip";
    undo.textContent = "Undo";
    undo.addEventListener("click", () => opts.undo?.());
    bar.appendChild(undo);
  }
  if (opts.confirm) {
    const confirm = document.createElement("button");
    confirm.type = "button";
    confirm.className = "confirm-targets-btn";
    confirm.textContent = opts.confirmLabel || "Confirm";
    confirm.addEventListener("click", () => opts.confirm?.());
    bar.appendChild(confirm);
  }
  if (opts.cancel) {
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "pending-cancel-chip";
    cancel.textContent = "Cancel";
    cancel.addEventListener("click", (e) => {
      e.stopPropagation();
      opts.cancel?.();
    });
    bar.appendChild(cancel);
  }
  document.body.appendChild(bar);
  watchPromptBar(bar);
}

function watchPromptBar(bar: HTMLElement): void {
  promptPlaceCleanup?.();
  const place = () => {
    if (!bar.isConnected) {
      promptPlaceCleanup?.();
      return;
    }
    placePromptBar(bar);
  };
  place();
  const ro = new ResizeObserver(place);
  for (const id of ["redBoard", "blueBoard", "appRoot"]) {
    const el = byId(id);
    if (el) ro.observe(el);
  }
  window.addEventListener("resize", place);
  promptPlaceCleanup = () => {
    ro.disconnect();
    window.removeEventListener("resize", place);
    promptPlaceCleanup = null;
  };
}

function placePromptBar(bar: HTMLElement): void {
  const red = byId("redBoard")?.getBoundingClientRect();
  const blue = byId("blueBoard")?.getBoundingClientRect();
  const play = byId("appRoot")?.getBoundingClientRect();
  if (!red || !blue || !play) return;
  const midY = (red.bottom + blue.top) / 2;
  const midX = (play.left + play.right) / 2;
  bar.style.top = `${midY}px`;
  bar.style.left = `${midX}px`;
  bar.style.bottom = "auto";
  bar.style.right = "auto";
  bar.style.transform = "translate(-50%, -50%)";
}

function paintChoicePrompt(
  text: string,
  hooks: RenderHooks,
  onConfirm: (() => void) | null,
  node: ChoiceNode,
): void {
  paintPromptBar({
    text,
    undo: () => hooks.onUndo(),
    confirm: onConfirm ?? undefined,
    confirmLabel: onConfirm ? confirmLabel(node) : undefined,
  });
}

function confirmLabel(node: ChoiceNode): string {
  return `${choicePrompt(node)} (${choiceSelectedCount(node)})`;
}

function choiceSelectedCount(node: ChoiceNode): number {
  if ("fuse_partners" in node) return node.fuse_partners.picked.length;
  if ("multi_pick" in node) return node.multi_pick.picked.length;
  if ("modes" in node) return node.modes.picked.length;
  if ("targets" in node) return Math.max(0, node.targets.options.length - node.targets.remaining);
  if ("cards" in node) return Math.max(0, node.cards.options.length - node.cards.remaining);
  return 0;
}

function choicePrompt(node: ChoiceNode): string {
  if ("fuse_partners" in node) return "Select a fuse partner";
  if ("modes" in node) return "Select a mode";
  if ("cards" in node) return "Select a card";
  if ("targets" in node) {
    const opts = node.targets.options;
    if (opts.some((o) => "hand" in o)) return "Select a card in your hand";
    if (opts.some((o) => "slot" in o && "player" in o)) return "Select a follower";
    if (opts.some((o) => "leader" in o)) return "Select a leader";
  }
  if ("multi_pick" in node) return "Select targets";
  return "Select a target";
}

function labelChooseAction(
  act: NeutralAction,
  node: ChoiceNode,
  full: FullState,
  acting: PlayerId,
): string {
  if (!("choose" in act)) return "Option";
  const o = act.choose.option;
  if (typeof o === "object" && o && "card" in o) return lookupText(o.card).name;
  if (typeof o === "object" && o && "mode" in o) {
    const last = full.players[acting].played_this_turn?.slice(-1)[0];
    const printed = last ? lookupText(last).modes?.[o.mode] : undefined;
    return printed || `Mode ${o.mode + 1}`;
  }
  if (o === "leader") return "Leader";
  if (typeof o === "object" && o && "slot" in o) return `Slot ${o.slot}`;
  void node;
  return "Option";
}

function nodeTargetOptions(node: ChoiceNode | null): TargetOpt[] {
  if (!node) return [];
  if ("targets" in node) return node.targets.options;
  if ("multi_pick" in node) return node.multi_pick.options;
  return [];
}

function slotOwnerFromNode(slot: number): PlayerId | undefined {
  const hits = nodeTargetOptions(currentChoiceNode).filter(
    (o): o is { slot: number; player: PlayerId } => "slot" in o && o.slot === slot,
  );
  if (hits.length === 1) return hits[0].player;
  return undefined;
}

function leaderOwnerFromNode(): PlayerId | undefined {
  const hits = nodeTargetOptions(currentChoiceNode).filter(
    (o): o is { leader: PlayerId } => "leader" in o,
  );
  if (hits.length === 1) return hits[0].leader;
  return undefined;
}

export function elForChooseOption(opt: unknown): HTMLElement | null {
  if (opt === "leader") {
    const owner = leaderOwnerFromNode();
    if (owner) return byId(`${visual(owner)}Leader`);
    return null;
  }
  if (!opt || typeof opt !== "object") return null;
  const o = opt as {
    slot?: number;
    player?: PlayerId;
    card?: string;
    hand?: { player: PlayerId; pos: number };
  };
  if (typeof o.slot === "number") {
    const player = o.player ?? slotOwnerFromNode(o.slot);
    if (!player) return null;
    return (
      byId(`${visual(player)}Board`)?.querySelector<HTMLElement>(`.card[data-slot="${o.slot}"]`) ??
      null
    );
  }
  if (o.hand) {
    return byId(`${visual(o.hand.player)}Hand`)?.querySelector(
      `.card[data-hand-pos="${o.hand.pos}"]`,
    ) as HTMLElement | null;
  }
  if (o.card) {
    return null;
  }
  if ("leader" in o && typeof (o as { leader?: PlayerId }).leader === "string") {
    return byId(`${visual((o as { leader: PlayerId }).leader)}Leader`);
  }
  return null;
}

export function chooseActionForElement(
  legal: NeutralAction[],
  el: HTMLElement,
): NeutralAction | null {
  const player = el.dataset.player as PlayerId | undefined;
  const slot = el.dataset.slot != null ? Number(el.dataset.slot) : undefined;
  const handPos = el.dataset.handPos != null ? Number(el.dataset.handPos) : undefined;
  const isLeader = el.classList.contains("leader-attack-strip") || el.id.endsWith("Leader");
  const leaderPlayer: PlayerId | undefined = isLeader
    ? el.id.startsWith("blue")
      ? "a"
      : "b"
    : undefined;
  let acting = currentChoicePlayer;
  if (!acting) {
    for (const a of legal) {
      if ("choose" in a) {
        acting = a.choose.player;
        break;
      }
    }
  }
  if (!acting) return null;

  if (typeof slot === "number" && player && el.closest(".board-zone")) {
    const nodeHit = nodeTargetOptions(currentChoiceNode).some(
      (o) => "slot" in o && o.slot === slot && o.player === player,
    );
    const legalHit = legal.some((a) => {
      if (!("choose" in a)) return false;
      const o = a.choose.option;
      if (!o || typeof o !== "object" || !("slot" in o)) return false;
      if (o.slot !== slot) return false;
      if (o.player && o.player !== player) return false;
      return true;
    });
    if (nodeHit || legalHit) {
      return { choose: { player: acting, option: { slot, player } } };
    }
    return null;
  }

  if (isLeader && leaderPlayer) {
    const nodeHit = nodeTargetOptions(currentChoiceNode).some(
      (o) => "leader" in o && o.leader === leaderPlayer,
    );
    const legalHit = legal.some((a) => "choose" in a && a.choose.option === "leader");
    if (nodeHit || legalHit) {
      return { choose: { player: acting, option: "leader" } };
    }
  }

  return (
    legal.find((a) => {
      if (!("choose" in a)) return false;
      const o = a.choose.option as
        | { card?: string; slot?: number; player?: PlayerId; hand?: { player: PlayerId; pos: number }; mode?: number }
        | "leader";
      if (o === "leader") return false;
      if (!o || typeof o !== "object") return false;
      if (o.hand) {
        return player === o.hand.player && handPos === o.hand.pos;
      }
      if (o.card) {
        return (
          !!player &&
          a.choose.player === player &&
          el.closest(".hand-zone") != null &&
          el.dataset.card === o.card
        );
      }
      if ("leader" in o) return leaderPlayer === (o as { leader: PlayerId }).leader;
      return false;
    }) ?? null
  );
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
      <div class="gameover-reason" id="gameOverReason"></div>
      <p class="gameover-hint">First player of a rematch follows the drawer’s A / B / coin setting.</p>
      <div class="gameover-actions">
        <button type="button" id="rematchSameSeedBtn">Rematch (same seed)</button>
        <button type="button" id="rematchNewSeedBtn">Rematch (new seed)</button>
        <button type="button" id="newGameFromOver">New Game</button>
      </div>
    </div>`;
    document.body.appendChild(overlay);
    overlay.querySelector("#newGameFromOver")?.addEventListener("click", () => hooks.onNewGame());
    overlay.querySelector("#rematchSameSeedBtn")?.addEventListener("click", () => hooks.onRematchSame());
    overlay.querySelector("#rematchNewSeedBtn")?.addEventListener("click", () => hooks.onRematchNew());
  }
  const title = document.getElementById("gameOverTitle");
  if (title) {
    title.textContent =
      full.winner === "a" ? "Blue (A) wins" : full.winner === "b" ? "Red (B) wins" : "Draw";
  }
  const reason = document.getElementById("gameOverReason");
  if (reason) {
    const loser = full.winner === "a" ? "b" : full.winner === "b" ? "a" : null;
    const deckout = loser != null && full.players[loser].deck.length === 0;
    reason.textContent = deckout ? "Deck-out" : "Lethal";
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
  paintPromptBar({
    text: pending.kind === "evolve" ? "Select a follower" : "Select a target",
    cancel: () => document.dispatchEvent(new CustomEvent("arena-cancel-pending")),
  });
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

function instForCard(card: HTMLElement, full: FullState | null): CardInstance | null {
  if (!full) return null;
  const uid = Number(card.dataset.uid);
  const player = card.dataset.player as PlayerId | undefined;
  if (!player || !uid) return null;
  const p = full.players[player];
  return p.hand.find((c) => c.id === uid) ?? p.field.find((c) => c?.id === uid) ?? null;
}

function paintTooltipHtml(card: HTMLElement, full: FullState | null): void {
  const tip = tipEl;
  if (!tip) return;
  const id = card.dataset.card;
  if (!id) return;
  const uid = Number(card.dataset.uid);
  const info = handInfoByUid.get(uid);
  const inst = instForCard(card, full);
  const player = card.dataset.player as PlayerId | undefined;
  const acting = full?.active;
  const blocked =
    player && acting === player && info && !info.playable ? info.blocked_reason : null;
  tip.innerHTML = formatCardTooltip({
    inst,
    cardId: id,
    gates: gateByUid.get(uid),
    displayCost: info?.cost ?? null,
    blockedReason: blocked ?? null,
    turn: full?.turn,
    rallyHave: player && full ? full.players[player].rally : undefined,
  });
  tip.style.display = "block";
}

function placeTooltip(clientX: number, clientY: number): void {
  const tip = tipEl;
  if (!tip) return;
  const left = Math.min(clientX + 12, window.innerWidth - tip.offsetWidth - 12);
  tip.style.left = `${Math.max(12, left)}px`;
  if (clientY > window.innerHeight / 2) {
    tip.style.bottom = `${window.innerHeight - clientY + 12}px`;
    tip.style.top = "auto";
  } else {
    tip.style.top = `${clientY + 12}px`;
    tip.style.bottom = "auto";
  }
}

function pinDragTooltip(): void {
  const tip = tipEl;
  if (!tip) return;
  tip.style.top = "12px";
  tip.style.bottom = "auto";
  tip.style.left = "12px";
}

export function beginDragTooltip(card: HTMLElement): void {
  if (card.dataset.faceDown === "1") return;
  tipSession = { card, drag: true };
  paintTooltipHtml(card, lastFull);
  pinDragTooltip();
}

export function endDragTooltip(): void {
  if (!tipSession?.drag) return;
  const hovered = document.querySelector<HTMLElement>(".card[data-card]:hover");
  if (hovered && hovered.dataset.faceDown !== "1") {
    tipSession = { card: hovered, drag: false };
    paintTooltipHtml(hovered, lastFull);
    return;
  }
  tipSession = null;
  if (tipEl) tipEl.style.display = "none";
}

function refreshOpenTooltip(full: FullState): void {
  if (!tipSession || !tipEl) return;
  if (!tipSession.card.isConnected) {
    tipSession = null;
    tipEl.style.display = "none";
    return;
  }
  paintTooltipHtml(tipSession.card, full);
  if (tipSession.drag) pinDragTooltip();
}

export function bindTooltips(): void {
  const tip = byId("cardTooltip");
  if (!tip) return;
  tipEl = tip;
  let pressTimer = 0;
  let pressCard: HTMLElement | null = null;
  let pressX = 0;
  let pressY = 0;
  let longPinned = false;

  document.addEventListener("mouseover", (e) => {
    if (tipSession?.drag) return;
    const card = (e.target as HTMLElement).closest<HTMLElement>(".card[data-card]");
    if (!card || card.dataset.faceDown === "1") return;
    tipSession = { card, drag: false };
    paintTooltipHtml(card, lastFull);
  });
  document.addEventListener("mousemove", (e) => {
    if (!tipEl || tipEl.style.display === "none") return;
    if (tipSession?.drag) {
      pinDragTooltip();
      return;
    }
    placeTooltip(e.clientX, e.clientY);
  });
  document.addEventListener("mouseout", (e) => {
    if (tipSession?.drag) return;
    const card = (e.target as HTMLElement).closest(".card[data-card]");
    if (!card) return;
    const next = (e.relatedTarget as HTMLElement | null)?.closest?.(".card[data-card]");
    if (next === card) return;
    if (tipSession?.card === card) tipSession = null;
    if (tipEl) tipEl.style.display = "none";
  });

  document.addEventListener(
    "pointerdown",
    (e) => {
      if (e.pointerType !== "touch") return;
      const card = (e.target as HTMLElement).closest<HTMLElement>(".card[data-card]");
      window.clearTimeout(pressTimer);
      pressCard = card;
      pressX = e.clientX;
      pressY = e.clientY;
      if (!card || card.dataset.faceDown === "1") return;
      pressTimer = window.setTimeout(() => {
        if (!pressCard) return;
        longPinned = true;
        tipSession = { card: pressCard, drag: false };
        paintTooltipHtml(pressCard, lastFull);
        placeTooltip(pressX, pressY);
      }, 400);
    },
    true,
  );
  document.addEventListener(
    "pointermove",
    (e) => {
      if (e.pointerType !== "touch" || !pressCard) return;
      const dx = e.clientX - pressX;
      const dy = e.clientY - pressY;
      if (dx * dx + dy * dy > 64) {
        window.clearTimeout(pressTimer);
        pressCard = null;
      }
    },
    true,
  );
  document.addEventListener(
    "pointerup",
    (e) => {
      window.clearTimeout(pressTimer);
      if (e.pointerType === "touch") {
        const card = (e.target as HTMLElement).closest<HTMLElement>(".card[data-card]");
        if (longPinned && (!card || card !== pressCard)) {
          longPinned = false;
          tipSession = null;
          if (tipEl) tipEl.style.display = "none";
        }
      }
      pressCard = null;
    },
    true,
  );
}
