import init, { botPolicies, bundleInfo, version } from "../pkg/arena_wasm.js";
import { publicUrl } from "./base.ts";
import { decks, loadCatalog, lookupText, parseDeckJson } from "./catalog.ts";
import { clearFloaters, reflashDamage, spawnFloaters } from "./fct.ts";
import { bindPointer } from "./input.ts";
import { sessionBoardInfo, sessionHandInfo, sessionPlayerInfo } from "./info.ts";
import * as L from "./legal.ts";
import {
  bindTooltips,
  render,
  resetZoneCache,
  type Pending,
  type RenderHooks,
} from "./render.ts";
import { byId } from "./render/ids.ts";
import {
  applyAction,
  botStep,
  canRedo,
  canUndo,
  createSession,
  disposeSession,
  isHumanActing,
  legalActions,
  redo,
  checkpointStatusText,
  replayPosition,
  rerollCheckpoint,
  restoreCheckpoint,
  setCheckpoint,
  toPositionLog,
  undo,
  type Session,
} from "./session.ts";
import { namedCounterValue, renderCard } from "./render/card.ts";
import { rerollSeed } from "./reroll.ts";
import { readShareParams, writeShareParams } from "./share.ts";
import type {
  EngineEvent,
  First,
  Mode,
  NeutralAction,
  PlayerId,
  PositionLog,
  SessionConfig,
} from "./types.ts";

let session: Session | null = null;
let pending: Pending = null;
let watchPlaying = false;
let watchTimer = 0;
let paintQueued = 0;
const importedDecks = new Map<string, { label: string; cards: Record<string, number> }>();
const savedPositions = new Map<string, PositionLog>();
let positionSeq = 0;
let toastTimer = 0;

const hooks: RenderHooks = {
  onPlay: (player, handPos) => {
    if (!session) return;
    const act = L.playAt(legalActions(session), player, handPos);
    if (act) commit(act);
  },
  onFuse: (player, handPos) => {
    if (!session) return;
    const act = L.fuseAt(legalActions(session), player, handPos);
    if (act) commit(act);
  },
  onEngage: (player, slot) => {
    if (!session) return;
    const act = L.engageAt(legalActions(session), player, slot);
    if (act) commit(act);
  },
  onAttack: (player, slot, target) => {
    if (!session) return;
    const act = legalActions(session).find(
      (a) =>
        "attack" in a &&
        a.attack.player === player &&
        a.attack.attacker_slot === slot &&
        JSON.stringify(a.attack.target) === JSON.stringify(target),
    );
    if (act) commit(act);
  },
  onEvolve: (player, slot, superEvo) => {
    if (!session) return;
    const act = legalActions(session).find(
      (a) =>
        "evolve" in a &&
        a.evolve.player === player &&
        a.evolve.slot === slot &&
        a.evolve.super === superEvo,
    );
    if (act) commit(act);
    pending = null;
  },
  onChooseOpt: (action) => commit(action),
  onConfirm: () => {
    if (!session) return;
    const act = L.confirm(legalActions(session));
    if (act) commit(act);
  },
  onMulliganToggle: (index) => {
    if (!session) return;
    session.mulliganSwap[index] = !session.mulliganSwap[index];
    paint();
  },
  onMulliganConfirm: (player) => {
    if (!session) return;
    const act = L.mulligan(legalActions(session), player, session.mulliganSwap);
    if (act) {
      session.mulliganSwap = [false, false, false, false];
      commit(act);
    }
  },
  onEndTurn: (player) => {
    if (!session) return;
    const act = L.endTurn(legalActions(session), player);
    if (act) commit(act);
  },
  onBonusPp: (player) => {
    if (!session) return;
    const act = L.bonusPp(legalActions(session), player);
    if (act) commit(act);
  },
  onNewGame: () => openSettingsForNewGame(),
  onRematchSame: () => void rematch(true),
  onRematchNew: () => void rematch(false),
  onUndo: () => applyHistory(undo),
  pending: null,
  setPending: (p) => {
    pending = p;
    requestPaint();
  },
};

function exposeArena(): void {
  window.__arena = {
    hash: () => session?.game.hash() ?? "",
    canUndo: () => (session ? canUndo(session) : false),
    canRedo: () => (session ? canRedo(session) : false),
    botAction: (policy, seed) => {
      if (!session) throw new Error("no session");
      return session.game.botAction(policy, seed);
    },
    apply: (actionJson) => {
      if (!session) throw new Error("no session");
      const action = (
        typeof actionJson === "string" ? JSON.parse(actionJson) : actionJson
      ) as NeutralAction;
      const events = applyAction(session, action);
      showCombat(events);
      return events;
    },
    handInfo: (player) => (session ? sessionHandInfo(session, player as PlayerId) : []),
    boardInfo: (player) => (session ? sessionBoardInfo(session, player as PlayerId) : []),
    playerInfo: (player) =>
      session
        ? sessionPlayerInfo(session, player as PlayerId)
        : {
            evolve_unlocked: false,
            super_evolve_unlocked: false,
            evolve_unlock_in: 0,
            super_evolve_unlock_in: 0,
            has_leader_barrier: false,
          },
    full: () => (session ? JSON.parse(session.game.full()) : null),
    legal: () => (session ? JSON.parse(session.game.legal()) : []),
    actions: () => (session ? session.actions : []),
    paintMs: window.__arena?.paintMs,
    watchDelayMs,
    humanSide: () => session?.cfg.humanSide ?? null,
    reseed: (seed) => {
      if (!session) throw new Error("no session");
      session.game.reseed(seed);
    },
    exportLog: () => (session ? toPositionLog(session) : null),
    loadLog: (log) => {
      loadLogSafely(log as PositionLog);
    },
    setCheckpoint: () => {
      if (!session) return;
      setCheckpoint(session);
      refreshCheckpointStatus();
    },
    restoreCheckpoint: () => {
      if (!session) return false;
      const ok = restoreCheckpoint(session);
      if (ok) {
        resetZoneCache();
        paint();
        refreshCheckpointStatus();
      }
      return ok;
    },
    reroll: () => {
      if (!session) return false;
      const ok = rerollCheckpoint(session);
      if (ok) {
        resetZoneCache();
        paint();
        refreshCheckpointStatus();
      }
      return ok;
    },
    rerollSeed: (seed, n) => rerollSeed(typeof seed === "bigint" ? seed : BigInt(seed), n).toString(),
    namedCounterValue,
    rematchSame: () => void rematch(true),
    savedPosition: () => {
      const id = byId<HTMLSelectElement>("positionSelect")?.value;
      return id ? (savedPositions.get(id) ?? null) : null;
    },
    debugGrantCantAttackLeader: (player, slot) => {
      if (!session) throw new Error("no session");
      session.game.debugGrantCantAttackLeader(player, slot);
      requestPaint();
    },
    mountNamedCounter: (vars) => {
      const host = document.getElementById("blueBoard") ?? document.body;
      const inst = {
        id: 9_900_001,
        card: "10031210",
        name: "Named counter",
        kind: "amulet",
        class: "runecraft",
        cost: 1,
        base_cost: 1,
        attack: 0,
        defense: 0,
        max_defense: 0,
        evolved: false,
        super_evolved: false,
        traits: [] as string[],
        printed_tags: [] as string[],
        granted: null,
        flags: {
          was_fused: false,
          fused_kinds: [] as string[],
          ambush_active: false,
          summoning_sick: false,
          attacked_this_turn: false,
          attacks_left: 0,
          engaged_this_turn: false,
          fused_this_turn: false,
          enhanced: false,
        },
        vars,
        skybound: 0,
        countdown: null,
        spellboost_count: 0,
        tribes: [] as string[],
      };
      const el = renderCard({ inst, elementId: "named-counter-demo", onBoard: true });
      host.appendChild(el);
      return el.id;
    },
  };
}

function paint(): void {
  if (!session) return;
  hooks.pending = pending;
  const counter = byId("turnCounter");
  if (counter) counter.dataset.botSeq = String(session.botSeq);
  document.body.dataset.watch = watchPlaying ? "1" : "0";
  render(session, hooks);
  exposeArena();
}

function requestPaint(): void {
  if (paintQueued) return;
  paintQueued = requestAnimationFrame(() => {
    paintQueued = 0;
    paintSafe();
  });
}

/** Undo / redo path — never calls maybeBots (replay uses stored snapshots). */
function applyHistory(fn: (s: Session) => boolean): void {
  if (!session) return;
  watchPlaying = false;
  window.clearTimeout(watchTimer);
  if (!fn(session)) return;
  clearFloaters();
  resetZoneCache();
  pending = null;
  paint();
}

function paintSafe(): void {
  try {
    paint();
  } catch (err) {
    console.error(err);
  }
}

function commit(action: NeutralAction): void {
  if (!session) return;
  try {
    const events = applyAction(session, action);
    showCombat(events);
    void maybeBots();
  } catch (err) {
    console.error(err);
    toast(String(err));
  }
}

function floatingTextOn(): boolean {
  const box = byId<HTMLInputElement>("floatingCombatTextToggle");
  return box ? box.checked : true;
}

function showCombat(events: EngineEvent[]): void {
  if (!session) return;
  const show = !session.suppressFloater && floatingTextOn();
  session.suppressFloater = false;
  pending = null;
  if (show) spawnFloaters(events, true);
  paint();
  if (show) reflashDamage(events);
}

function toast(msg: string): void {
  let pill = byId("actionToast");
  if (!pill) {
    pill = document.createElement("div");
    pill.id = "actionToast";
    pill.setAttribute("aria-live", "polite");
    document.body.appendChild(pill);
  }
  window.clearTimeout(toastTimer);
  if (!msg) {
    pill.classList.remove("visible");
    pill.textContent = "";
    return;
  }
  pill.textContent = msg;
  pill.classList.add("visible");
  toastTimer = window.setTimeout(() => pill.classList.remove("visible"), 1800);
}

function humanSideSelectValue(): "a" | "b" | "coin" {
  const v = byId<HTMLSelectElement>("humanSideSelect")?.value ?? "a";
  if (v === "b" || v === "coin") return v;
  return "a";
}

/** Roll coin only here — never from label or policy-sync listeners. */
function rollHumanSide(): PlayerId {
  const v = humanSideSelectValue();
  if (v === "b") return "b";
  if (v === "coin") return Math.random() < 0.5 ? "a" : "b";
  return "a";
}

/** Side used for vs-bot labels and bot-policy sync. Coin does not re-roll. */
function uiHumanSide(): { side: PlayerId; coinPending: boolean } {
  const v = humanSideSelectValue();
  if (v === "a") return { side: "a", coinPending: false };
  if (v === "b") return { side: "b", coinPending: false };
  if (session) return { side: session.cfg.humanSide, coinPending: false };
  return { side: "a", coinPending: true };
}

function formConfig(): SessionConfig {
  const seedRaw = byId<HTMLInputElement>("seedInput")?.value.trim() ?? "";
  const seed = seedRaw ? BigInt(seedRaw) : randomSeed();
  const deckAId = byId<HTMLSelectElement>("blueDeckSelect")!.value;
  const deckBId = byId<HTMLSelectElement>("redDeckSelect")!.value;
  const mode = (byId<HTMLSelectElement>("modeSelect")?.value ?? "hotseat") as Mode;
  const first = (byId<HTMLSelectElement>("firstSelect")?.value ?? "coin") as First;
  return {
    seed,
    deckA: deckCardsResolved(deckAId),
    deckB: deckCardsResolved(deckBId),
    deckAId,
    deckBId,
    first,
    mode,
    humanSide: rollHumanSide(),
    hideBotHand: byId<HTMLInputElement>("hideBotHandToggle")?.checked ?? true,
    policyA: byId<HTMLSelectElement>("policyASelect")?.value ?? "random",
    policyB: byId<HTMLSelectElement>("policyBSelect")?.value ?? "random",
  };
}

const deckCache = new Map<string, Record<string, number>>();

async function ensureDecks(): Promise<void> {
  for (const d of decks()) {
    if (deckCache.has(d.id)) continue;
    const raw = await fetch(publicUrl(`decks/${d.file}`)).then((r) => r.text());
    deckCache.set(d.id, parseDeckJson(raw));
  }
}

function deckCardsResolved(id: string): Record<string, number> {
  return importedDecks.get(id)?.cards ?? deckCache.get(id) ?? {};
}

async function startFromForm(): Promise<void> {
  await ensureDecks();
  const cfg = formConfig();
  cfg.deckA = deckCardsResolved(cfg.deckAId);
  cfg.deckB = deckCardsResolved(cfg.deckBId);
  startSession(cfg);
}

function startSession(cfg: SessionConfig): void {
  watchPlaying = false;
  disposeSession(session);
  session = createSession(cfg);
  resetZoneCache();
  closeSettings();
  writeShareParams({
    seed: cfg.seed.toString(),
    deckA: cfg.deckAId,
    deckB: cfg.deckBId,
    mode: cfg.mode,
  });
  const seedPanel = byId("gameSeedPanel");
  const seedVal = byId("gameSeedValue");
  if (seedPanel && seedVal) {
    seedPanel.hidden = false;
    seedVal.textContent = cfg.seed.toString();
  }
  toast("");
  pending = null;
  refreshCheckpointStatus();
  paint();
  syncVsBotRoleLabels();
  void maybeBots();
  if (cfg.mode === "watch") startWatchIfAuto();
}

function startWatchIfAuto(): void {
  const auto = byId<HTMLInputElement>("watchAutoStart");
  if (auto?.checked) {
    watchPlaying = true;
    scheduleWatch();
  }
}

function randomSeed(): bigint {
  const buf = new Uint32Array(2);
  crypto.getRandomValues(buf);
  return (BigInt(buf[0]) << 32n) | BigInt(buf[1]);
}

function openSettingsForNewGame(): void {
  closeHistory();
  const drawer = byId("settingsDrawer");
  const scrim = byId("settingsScrim");
  drawer?.classList.add("open");
  drawer?.setAttribute("aria-hidden", "false");
  scrim?.classList.add("show");
}

/** Terminal overlay "Rematch (same seed)" / "Rematch (new seed)". */
async function rematch(keepSeed: boolean): Promise<void> {
  if (!session) return;
  const cfg = { ...session.cfg };
  cfg.first = (byId<HTMLSelectElement>("firstSelect")?.value ?? cfg.first) as First;
  if (!keepSeed) {
    cfg.seed = randomSeed();
    const seedInput = byId<HTMLInputElement>("seedInput");
    if (seedInput) seedInput.value = cfg.seed.toString();
  }
  startSession(cfg);
}

async function maybeBots(): Promise<void> {
  if (!session) return;
  if (session.game.phase() === "terminal") return;
  if (session.cfg.mode === "hotseat") return;
  if (session.cfg.mode === "watch") {
    if (watchPlaying) scheduleWatch();
    return;
  }
  // vs-bot — one engine action per beat so the human can follow.
  let guard = 0;
  while (session && !isHumanActing(session) && session.game.phase() !== "terminal" && guard < 80) {
    const events = botStep(session);
    guard += 1;
    showCombat(events);
    await new Promise<void>((r) => window.setTimeout(r, 280));
  }
  paint();
}

/** Geometric watch delay: v1 = 3000 ms … v19 = 16 ms, v20 = 0 (unthrottled). */
const WATCH_DELAY_R = (16 / 3000) ** (1 / 18);

function watchDelayMsFromValue(v: number): number {
  const n = Math.round(Number(v));
  if (!Number.isFinite(n) || n >= 20) return 0;
  if (n <= 1) return 3000;
  return Math.round(3000 * WATCH_DELAY_R ** (n - 1));
}

function watchDelayMs(): number {
  const sl = byId<HTMLInputElement>("watchSpeed");
  return watchDelayMsFromValue(sl ? Number(sl.value) : 5);
}

function formatWatchSpeedLabel(delay: number): string {
  if (delay <= 0) return "max";
  const sec = delay / 1000;
  if (delay >= 1000) return `${sec.toFixed(1)} s / action`;
  return `${sec.toFixed(2)} s / action`;
}

function syncWatchSpeedReadout(): void {
  const out = byId("watchSpeedReadout");
  if (out) out.textContent = formatWatchSpeedLabel(watchDelayMs());
}

function scheduleWatch(): void {
  window.clearTimeout(watchTimer);
  if (!watchPlaying || !session) return;
  const delay = watchDelayMs();
  const step = () => {
    if (!watchPlaying || !session || session.game.phase() === "terminal") {
      watchPlaying = false;
      paint();
      return;
    }
    try {
      const events = botStep(session);
      resetZoneCache();
      showCombat(events);
    } catch (err) {
      watchPlaying = false;
      toast(String(err));
      console.error(err);
      paintSafe();
      return;
    }
    if (watchPlaying) {
      if (delay === 0) requestAnimationFrame(() => scheduleWatch());
      else scheduleWatch();
    }
  };
  if (delay === 0) requestAnimationFrame(step);
  else watchTimer = window.setTimeout(step, delay);
}

function populatePolicies(): void {
  const names = JSON.parse(botPolicies()) as string[];
  for (const id of ["policyASelect", "policyBSelect", "vsBotPolicy"]) {
    const sel = byId<HTMLSelectElement>(id);
    if (!sel) continue;
    sel.innerHTML = "";
    for (const n of names) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      sel.appendChild(o);
    }
  }
  byId("vsBotPolicy")?.addEventListener("change", applyVsBotPolicy);
}

function applyVsBotPolicy(): void {
  const vs = byId<HTMLSelectElement>("vsBotPolicy");
  const a = byId<HTMLSelectElement>("policyASelect");
  const b = byId<HTMLSelectElement>("policyBSelect");
  if (!vs || !a || !b) return;
  // Human is one side; bot policy applies to the other. Coin does not re-roll.
  if (uiHumanSide().side === "a") b.value = vs.value;
  else a.value = vs.value;
}

function populateDecks(): void {
  const share = readShareParams();
  for (const id of ["blueDeckSelect", "redDeckSelect"]) {
    const sel = byId<HTMLSelectElement>(id);
    if (!sel) continue;
    sel.innerHTML = "";
    for (const d of decks()) {
      const o = document.createElement("option");
      o.value = d.id;
      o.textContent = d.label;
      sel.appendChild(o);
    }
    for (const [impId, rec] of importedDecks) {
      const o = document.createElement("option");
      o.value = impId;
      o.textContent = rec.label;
      sel.appendChild(o);
    }
  }
  const a = byId<HTMLSelectElement>("blueDeckSelect");
  const b = byId<HTMLSelectElement>("redDeckSelect");
  if (a) a.value = share.deckA && optionExists(a, share.deckA) ? share.deckA : "basic-forest";
  if (b) b.value = share.deckB && optionExists(b, share.deckB) ? share.deckB : "basic-rune";
  const seed = byId<HTMLInputElement>("seedInput");
  if (seed && share.seed) seed.value = share.seed;
  const mode = byId<HTMLSelectElement>("modeSelect");
  if (mode && share.mode) mode.value = share.mode;
}

function optionExists(sel: HTMLSelectElement, value: string): boolean {
  return [...sel.options].some((o) => o.value === value);
}

function syncModeChrome(): void {
  const mode = (byId<HTMLSelectElement>("modeSelect")?.value ?? "hotseat") as Mode;
  document.body.classList.toggle("mode-watch", mode === "watch");
  document.body.classList.toggle("mode-vs-bot", mode === "vs-bot");
  byId("vsBotFields")?.toggleAttribute("hidden", mode !== "vs-bot");
  byId("watchFields")?.toggleAttribute("hidden", mode !== "watch");
  syncVsBotRoleLabels();
}

function syncVsBotRoleLabels(): void {
  const blue = byId("blueDeckLabel");
  const red = byId("redDeckLabel");
  const policy = byId("vsBotPolicyLabel");
  const mode = (byId<HTMLSelectElement>("modeSelect")?.value ?? "hotseat") as Mode;
  if (!blue || !red) return;
  if (mode !== "vs-bot") {
    blue.textContent = "Blue (A):";
    red.textContent = "Red (B):";
    if (policy) policy.textContent = "Bot policy";
    return;
  }
  const { side, coinPending } = uiHumanSide();
  if (coinPending) {
    blue.textContent = "Your deck (coin — decided at start)";
    red.textContent = "Bot deck (Red B)";
    if (policy) policy.textContent = "Bot policy (Red B)";
    return;
  }
  if (side === "a") {
    blue.textContent = "Your deck (Blue A)";
    red.textContent = "Bot deck (Red B)";
    if (policy) policy.textContent = "Bot policy (Red B)";
  } else {
    blue.textContent = "Bot deck (Blue A)";
    red.textContent = "Your deck (Red B)";
    if (policy) policy.textContent = "Bot policy (Blue A)";
  }
}

function closeSettings(): void {
  const drawer = byId("settingsDrawer");
  const scrim = byId("settingsScrim");
  drawer?.classList.remove("open");
  drawer?.setAttribute("aria-hidden", "true");
  scrim?.classList.remove("show");
}

function initSettings(): void {
  const toggle = byId("settingsToggle");
  const drawer = byId("settingsDrawer");
  const scrim = byId("settingsScrim");
  const open = () => {
    closeHistory();
    drawer?.classList.add("open");
    drawer?.setAttribute("aria-hidden", "false");
    scrim?.classList.add("show");
  };
  const close = () => closeSettings();
  toggle?.addEventListener("click", (e) => {
    e.stopPropagation();
    drawer?.classList.contains("open") ? close() : open();
  });
  scrim?.addEventListener("click", close);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      const hist = byId("historyDrawer");
      if (hist?.classList.contains("open")) {
        closeHistory();
        return;
      }
      close();
    }
    if (e.key === "m" && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      drawer?.classList.contains("open") ? close() : open();
    }
  });
}

function initHistory(): void {
  const toggle = byId("historyToggle");
  const drawer = byId("historyDrawer");
  const scrim = byId("historyScrim");
  toggle?.addEventListener("click", (e) => {
    e.stopPropagation();
    if (drawer?.classList.contains("open")) closeHistory();
    else {
      byId("settingsDrawer")?.classList.remove("open");
      drawer?.classList.add("open");
      drawer?.setAttribute("aria-hidden", "false");
      scrim?.classList.add("show");
    }
  });
  scrim?.addEventListener("click", closeHistory);
}

function closeHistory(): void {
  byId("historyDrawer")?.classList.remove("open");
  byId("historyDrawer")?.setAttribute("aria-hidden", "true");
  byId("historyScrim")?.classList.remove("show");
}

function initHotkeys(): void {
  document.addEventListener("keydown", (e) => {
    const el = e.target as HTMLElement | null | undefined;
    if (el) {
      const tag = (el.tagName || "").toUpperCase();
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || el.isContentEditable) {
        return;
      }
    }
    const isMac =
      typeof navigator !== "undefined" &&
      navigator.platform &&
      navigator.platform.toUpperCase().includes("MAC");
    const ctrl = isMac ? e.metaKey : e.ctrlKey;
    if (ctrl && !e.shiftKey && (e.key === "z" || e.key === "Z")) {
      e.preventDefault();
      applyHistory(undo);
      return;
    }
    if (
      (ctrl && e.shiftKey && (e.key === "z" || e.key === "Z")) ||
      (e.ctrlKey && (e.key === "y" || e.key === "Y"))
    ) {
      e.preventDefault();
      applyHistory(redo);
      return;
    }
    if (e.key === "F6") {
      e.preventDefault();
      if (session) {
        setCheckpoint(session);
        refreshCheckpointStatus();
      }
    }
    if (e.key === "F7") {
      e.preventDefault();
      if (session && restoreCheckpoint(session)) {
        resetZoneCache();
        paint();
        refreshCheckpointStatus();
      }
    }
    if (e.key === "F8") {
      e.preventDefault();
      if (session && rerollCheckpoint(session)) {
        resetZoneCache();
        paint();
        refreshCheckpointStatus();
      }
    }
  });
}

function refreshCheckpointStatus(): void {
  setText("checkpointStatus", checkpointStatusText(session));
}

function setText(id: string, text: string): void {
  const el = byId(id);
  if (el) el.textContent = text;
}

function loadLogSafely(log: PositionLog): void {
  let next: Session | null = null;
  try {
    next = replayPosition(log);
  } catch (err) {
    toast(String(err));
    return;
  }
  disposeSession(session);
  session = next;
  pending = null;
  resetZoneCache();
  paint();
}

function initPositions(): void {
  byId("savePositionBtn")?.addEventListener("click", () => {
    if (!session) return;
    const name = window.prompt("Position name", `pos-${savedPositions.size + 1}`);
    if (!name) return;
    const id = `pos-${++positionSeq}`;
    savedPositions.set(id, toPositionLog(session, { name, savedAt: new Date().toISOString() }));
    refreshPositionSelect(id);
  });
  byId("loadPositionBtn")?.addEventListener("click", () => {
    const id = byId<HTMLSelectElement>("positionSelect")?.value;
    if (!id) return;
    const log = savedPositions.get(id);
    if (!log) return;
    loadLogSafely(log);
  });
  byId("exportPositionBtn")?.addEventListener("click", () => {
    const id = byId<HTMLSelectElement>("positionSelect")?.value;
    const log = (id && savedPositions.get(id)) || (session ? toPositionLog(session) : null);
    if (!log) return;
    const blob = new Blob([JSON.stringify(log, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${log.name || id || "position"}.json`;
    a.click();
  });
  byId("importPositionBtn")?.addEventListener("click", () => {
    byId<HTMLInputElement>("importPositionInput")?.click();
  });
  byId<HTMLInputElement>("importPositionInput")?.addEventListener("change", async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    try {
      const log = JSON.parse(await file.text()) as PositionLog;
      if (!log.name) log.name = file.name.replace(/\.json$/i, "");
      const id = `pos-${++positionSeq}`;
      savedPositions.set(id, log);
      refreshPositionSelect(id);
      loadLogSafely(log);
    } catch (err) {
      toast(String(err));
    }
  });
  byId("renamePositionBtn")?.addEventListener("click", () => {
    const sel = byId<HTMLSelectElement>("positionSelect");
    const id = sel?.value;
    if (!id) return;
    const log = savedPositions.get(id);
    if (!log) return;
    const name = window.prompt("Rename position", log.name || id);
    if (!name) return;
    log.name = name;
    refreshPositionSelect(id);
  });
  byId("deletePositionBtn")?.addEventListener("click", () => {
    const sel = byId<HTMLSelectElement>("positionSelect");
    const id = sel?.value;
    if (!id || !savedPositions.has(id)) return;
    savedPositions.delete(id);
    refreshPositionSelect();
  });
  byId("setCheckpointBtn")?.addEventListener("click", () => {
    if (!session) return;
    setCheckpoint(session);
    refreshCheckpointStatus();
  });
  byId("restoreCheckpointBtn")?.addEventListener("click", () => {
    if (!session) return;
    restoreCheckpoint(session);
    resetZoneCache();
    paint();
    refreshCheckpointStatus();
  });
  byId("rerollBtn")?.addEventListener("click", () => {
    if (!session) return;
    rerollCheckpoint(session);
    resetZoneCache();
    paint();
    refreshCheckpointStatus();
  });
}

function formatPositionOption(log: PositionLog): string {
  const name = log.name || "position";
  const turn = log.turn ?? "?";
  const time = log.savedAt ? new Date(log.savedAt).toLocaleTimeString() : "";
  return time ? `${name} · T${turn} · ${time}` : `${name} · T${turn}`;
}

function refreshPositionSelect(keep?: string): void {
  const sel = byId<HTMLSelectElement>("positionSelect");
  if (!sel) return;
  sel.innerHTML = "";
  if (!savedPositions.size) {
    const o = document.createElement("option");
    o.value = "";
    o.textContent = "(no saved positions)";
    sel.appendChild(o);
    sel.disabled = true;
    return;
  }
  sel.disabled = false;
  for (const [id, log] of savedPositions) {
    const o = document.createElement("option");
    o.value = id;
    o.textContent = formatPositionOption(log);
    sel.appendChild(o);
  }
  if (keep && savedPositions.has(keep)) sel.value = keep;
}

function blueDeckListText(): string {
  if (!session) return "";
  const lines: string[] = [];
  for (const [id, n] of Object.entries(session.cfg.deckA)) {
    const name = lookupText(id).name || id;
    lines.push(`${n}x ${name}`);
  }
  return lines.join("\n");
}

function initExportList(): void {
  byId("exportListBtn")?.addEventListener("click", async () => {
    const text = blueDeckListText();
    const panel = byId("exportListPanel");
    if (panel) {
      panel.hidden = !text;
      panel.textContent = text;
    }
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      /* panel still shows the list */
    }
  });
}

function initImportDeck(): void {
  byId("importDeckBtn")?.addEventListener("click", () => {
    byId<HTMLInputElement>("deckImportFileInput")?.click();
  });
  byId<HTMLInputElement>("deckImportFileInput")?.addEventListener("change", async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const cards = parseDeckJson(await file.text());
    const id = `import-${file.name.replace(/\.json$/i, "")}`;
    importedDecks.set(id, { label: `${file.name} (import)`, cards });
    populateDecks();
    const a = byId<HTMLSelectElement>("blueDeckSelect");
    if (a) a.value = id;
  });
}

function initWatch(): void {
  const sl = byId<HTMLInputElement>("watchSpeed");
  sl?.addEventListener("input", () => {
    localStorage.setItem("svwb.watchSpeed", sl.value);
    syncWatchSpeedReadout();
  });
  sl?.addEventListener("change", () => {
    localStorage.setItem("svwb.watchSpeed", sl.value);
    syncWatchSpeedReadout();
  });
  byId("watchStepBtn")?.addEventListener("click", () => {
    if (!session || session.cfg.mode !== "watch") return;
    watchPlaying = false;
    try {
      const events = botStep(session);
      resetZoneCache();
      showCombat(events);
    } catch (err) {
      toast(String(err));
      paint();
    }
  });
  byId("watchPlayBtn")?.addEventListener("click", () => {
    watchPlaying = true;
    scheduleWatch();
  });
  byId("watchPauseBtn")?.addEventListener("click", () => {
    watchPlaying = false;
  });
}

function restorePersistedToggles(): void {
  const bottom = localStorage.getItem("svwb.activeOnBottom") === "1";
  const fct = localStorage.getItem("svwb.floatingCombatText");
  const bottomBox = byId<HTMLInputElement>("activeOnBottomToggle");
  const fctBox = byId<HTMLInputElement>("floatingCombatTextToggle");
  if (bottomBox) bottomBox.checked = bottom;
  document.body.classList.toggle("active-on-bottom", bottom);
  if (fctBox) fctBox.checked = fct == null ? true : fct !== "0";
  const speedRaw = localStorage.getItem("svwb.watchSpeed");
  const sl = byId<HTMLInputElement>("watchSpeed");
  if (sl && speedRaw != null) {
    const n = Math.round(Number(speedRaw));
    if (Number.isFinite(n) && n >= 1 && n <= 20) sl.value = String(n);
  }
  syncWatchSpeedReadout();
}

async function boot(): Promise<void> {
  restorePersistedToggles();
  await init();
  await loadCatalog();
  populatePolicies();
  populateDecks();
  syncModeChrome();
  initSettings();
  initHistory();
  initHotkeys();
  initPositions();
  initExportList();
  initImportDeck();
  initWatch();
  bindTooltips();
  bindPointer({
    play: (p, i) => hooks.onPlay(p, i),
    attack: (p, s, t) => hooks.onAttack(p, s, t),
    evolve: (p, s, ev) => hooks.onEvolve(p, s, ev),
    engage: (p, s) => hooks.onEngage(p, s),
    fuse: (p, i) => hooks.onFuse(p, i),
    mulliganToggle: (i) => hooks.onMulliganToggle(i),
    choose: (a) => hooks.onChooseOpt(a),
    getLegal: () => (session ? legalActions(session) : []),
    getPhase: () => session?.game.phase() ?? "",
    getActing: () => (session ? (session.game.acting() as PlayerId) : null),
    getPending: () => pending,
    setPending: (p) => {
      pending = p;
      requestPaint();
    },
    cancelPending: () => {
      if (!pending) return;
      pending = null;
      requestPaint();
    },
  });
  byId("bonusPpBtn")?.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (!session) return;
    const full = JSON.parse(session.game.full()) as { players: { a: { is_second: boolean } } };
    const second: PlayerId = full.players.a.is_second ? "a" : "b";
    const act = L.bonusPp(legalActions(session), second);
    if (act) commit(act);
  });

  byId("startGameBtn")?.addEventListener("click", () => void startFromForm());
  byId("undoBtn")?.addEventListener("click", () => applyHistory(undo));
  byId("redoBtn")?.addEventListener("click", () => applyHistory(redo));
  exposeArena();
  byId("modeSelect")?.addEventListener("change", syncModeChrome);
  byId("humanSideSelect")?.addEventListener("change", () => {
    syncVsBotRoleLabels();
    applyVsBotPolicy();
  });
  byId("activeOnBottomToggle")?.addEventListener("change", (e) => {
    const on = (e.target as HTMLInputElement).checked;
    document.body.classList.toggle("active-on-bottom", on);
    localStorage.setItem("svwb.activeOnBottom", on ? "1" : "0");
  });
  byId("floatingCombatTextToggle")?.addEventListener("change", (e) => {
    const on = (e.target as HTMLInputElement).checked;
    localStorage.setItem("svwb.floatingCombatText", on ? "1" : "0");
  });
  byId("copySeedBtn")?.addEventListener("click", async () => {
    const btn = byId<HTMLButtonElement>("copySeedBtn");
    const v = byId("gameSeedValue")?.textContent ?? "";
    try {
      await navigator.clipboard.writeText(v);
    } catch {
      toast(v);
    }
    if (btn) {
      const prev = btn.textContent;
      btn.textContent = "Copied";
      window.setTimeout(() => {
        btn.textContent = prev || "Copy";
      }, 1200);
    }
  });
  byId("vsBotPolicy")?.addEventListener("change", applyVsBotPolicy);

  const info = JSON.parse(bundleInfo()) as {
    cards: number;
    crests: number;
    bytes: number;
  };
  const meta = byId("bundleMeta");
  if (meta) meta.textContent = `arena ${version()} · ${info.cards} cards`;

  const share = readShareParams();
  if (share.seed && share.deckA && share.deckB) {
    void startFromForm();
  }
}

void boot();
