import init, { botPolicies, bundleInfo, version } from "../pkg/arena_wasm.js";
import { decks, loadCatalog, parseDeckJson } from "./catalog.ts";
import { clearFloaters, spawnFloaters } from "./fct.ts";
import { bindPointer } from "./input.ts";
import * as L from "./legal.ts";
import { bindTooltips, render, resetZoneCache, type RenderHooks } from "./render.ts";
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
  replayPosition,
  restoreCheckpoint,
  setCheckpoint,
  toPositionLog,
  undo,
  type Session,
} from "./session.ts";
import { readShareParams, writeShareParams } from "./share.ts";
import type { First, Mode, NeutralAction, PlayerId, PositionLog, SessionConfig } from "./types.ts";

let session: Session | null = null;
let evoArmed: { player: PlayerId; superEvo: boolean } | null = null;
let watchPlaying = false;
let watchTimer = 0;
const importedDecks = new Map<string, { label: string; cards: Record<string, number> }>();
const savedPositions = new Map<string, PositionLog>();

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
    evoArmed = null;
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
  onNewGame: () => void startFromForm(),
  onRematchSwap: () => void rematchSwap(),
  evoArmed: null,
};

function exposeArena(): void {
  window.__arena = {
    hash: () => session?.game.hash() ?? "",
    canUndo: () => (session ? canUndo(session) : false),
    canRedo: () => (session ? canRedo(session) : false),
  };
}

function paint(): void {
  if (!session) return;
  hooks.evoArmed = evoArmed;
  const counter = byId("turnCounter");
  if (counter) counter.dataset.botSeq = String(session.botSeq);
  document.body.dataset.watch = watchPlaying ? "1" : "0";
  render(session, hooks);
  exposeArena();
}

/** Undo / redo path — never calls maybeBots (replay uses stored snapshots). */
function applyHistory(fn: (s: Session) => boolean): void {
  if (!session) return;
  watchPlaying = false;
  window.clearTimeout(watchTimer);
  if (!fn(session)) return;
  clearFloaters();
  resetZoneCache();
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
    if (!session.suppressFloater) {
      spawnFloaters(events, floatingTextOn());
    }
    session.suppressFloater = false;
    resetZoneCache();
    paint();
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

function toast(msg: string): void {
  const err = byId("errBanner");
  if (err) err.textContent = msg;
}

function humanSideFromForm(): PlayerId {
  const v = (byId<HTMLSelectElement>("humanSideSelect")?.value ?? "a") as string;
  if (v === "b") return "b";
  if (v === "coin") return Math.random() < 0.5 ? "a" : "b";
  return "a";
}

function formConfig(): SessionConfig {
  const seedRaw = byId<HTMLInputElement>("seedInput")?.value.trim() || "1";
  const seed = BigInt(seedRaw);
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
    humanSide: humanSideFromForm(),
    hideBotHand: byId<HTMLInputElement>("hideBotHandToggle")?.checked ?? true,
    policyA: byId<HTMLSelectElement>("policyASelect")?.value ?? "random",
    policyB: byId<HTMLSelectElement>("policyBSelect")?.value ?? "random",
  };
}

const deckCache = new Map<string, Record<string, number>>();

async function ensureDecks(): Promise<void> {
  for (const d of decks()) {
    if (deckCache.has(d.id)) continue;
    const raw = await fetch(`/decks/${d.file}`).then((r) => r.text());
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
  paint();
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

async function rematchSwap(): Promise<void> {
  if (!session) return;
  const cfg = { ...session.cfg };
  const da = cfg.deckA;
  cfg.deckA = cfg.deckB;
  cfg.deckB = da;
  const ida = cfg.deckAId;
  cfg.deckAId = cfg.deckBId;
  cfg.deckBId = ida;
  if (cfg.mode === "vs-bot") cfg.humanSide = cfg.humanSide === "a" ? "b" : "a";
  const aSel = byId<HTMLSelectElement>("blueDeckSelect");
  const bSel = byId<HTMLSelectElement>("redDeckSelect");
  if (aSel) aSel.value = cfg.deckAId;
  if (bSel) bSel.value = cfg.deckBId;
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
  // vs-bot
  let guard = 0;
  while (session && !isHumanActing(session) && session.game.phase() !== "terminal" && guard < 80) {
    const events = botStep(session);
    if (!session.suppressFloater) spawnFloaters(events, floatingTextOn());
    session.suppressFloater = false;
    guard += 1;
    if (guard % 4 === 0) {
      resetZoneCache();
      paint();
      await new Promise<void>((r) => requestAnimationFrame(() => r()));
    }
  }
  resetZoneCache();
  paint();
}

function watchDelayMs(): number {
  const sl = byId<HTMLInputElement>("watchSpeed");
  const v = sl ? Number(sl.value) : 1;
  if (v >= 20) return 0;
  return Math.max(0, Math.round(1000 / v));
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
      if (!session.suppressFloater) spawnFloaters(events, floatingTextOn());
      session.suppressFloater = false;
      resetZoneCache();
      paint();
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
  const vs = byId<HTMLSelectElement>("vsBotPolicy");
  const a = byId<HTMLSelectElement>("policyASelect");
  const b = byId<HTMLSelectElement>("policyBSelect");
  vs?.addEventListener("change", () => {
    if (!a || !b || !vs) return;
    // Human is one side; bot policy applies to the other. Both selectors
    // stay in sync so Watch can still pick per-side.
    if (humanSideFromForm() === "a") b.value = vs.value;
    else a.value = vs.value;
  });
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
    if (e.key === "Escape") close();
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
        setText("checkpointStatus", "Checkpoint: set");
      }
    }
    if (e.key === "F7") {
      e.preventDefault();
      if (session && restoreCheckpoint(session)) {
        resetZoneCache();
        paint();
      }
    }
  });
}

function setText(id: string, text: string): void {
  const el = byId(id);
  if (el) el.textContent = text;
}

function initPositions(): void {
  byId("savePositionBtn")?.addEventListener("click", () => {
    if (!session) return;
    const name = window.prompt("Position name", `pos-${savedPositions.size + 1}`);
    if (!name) return;
    savedPositions.set(name, toPositionLog(session));
    refreshPositionSelect();
  });
  byId("loadPositionBtn")?.addEventListener("click", () => {
    const id = byId<HTMLSelectElement>("positionSelect")?.value;
    if (!id) return;
    const log = savedPositions.get(id);
    if (!log) return;
    disposeSession(session);
    session = replayPosition(log);
    resetZoneCache();
    paint();
  });
  byId("exportPositionBtn")?.addEventListener("click", () => {
    const id = byId<HTMLSelectElement>("positionSelect")?.value;
    const log = (id && savedPositions.get(id)) || (session ? toPositionLog(session) : null);
    if (!log) return;
    const blob = new Blob([JSON.stringify(log, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${id || "position"}.json`;
    a.click();
  });
  byId("importPositionBtn")?.addEventListener("click", () => {
    byId<HTMLInputElement>("importPositionInput")?.click();
  });
  byId<HTMLInputElement>("importPositionInput")?.addEventListener("change", async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const log = JSON.parse(await file.text()) as PositionLog;
    savedPositions.set(file.name.replace(/\.json$/i, ""), log);
    refreshPositionSelect();
    disposeSession(session);
    session = replayPosition(log);
    resetZoneCache();
    paint();
  });
  byId("setCheckpointBtn")?.addEventListener("click", () => {
    if (!session) return;
    setCheckpoint(session);
    setText("checkpointStatus", "Checkpoint: set");
  });
  byId("restoreCheckpointBtn")?.addEventListener("click", () => {
    if (!session) return;
    restoreCheckpoint(session);
    resetZoneCache();
    paint();
  });
}

function refreshPositionSelect(): void {
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
  for (const name of savedPositions.keys()) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = name;
    sel.appendChild(o);
  }
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
  byId("watchStepBtn")?.addEventListener("click", () => {
    if (!session || session.cfg.mode !== "watch") return;
    watchPlaying = false;
    try {
      const events = botStep(session);
      spawnFloaters(events, floatingTextOn());
    } catch (err) {
      toast(String(err));
    }
    resetZoneCache();
    paint();
  });
  byId("watchPlayBtn")?.addEventListener("click", () => {
    watchPlaying = true;
    scheduleWatch();
  });
  byId("watchPauseBtn")?.addEventListener("click", () => {
    watchPlaying = false;
  });
}

async function boot(): Promise<void> {
  await init();
  await loadCatalog();
  populatePolicies();
  populateDecks();
  syncModeChrome();
  initSettings();
  initHistory();
  initHotkeys();
  initPositions();
  initImportDeck();
  initWatch();
  bindTooltips();
  bindPointer({
    legal: () => (session ? legalActions(session) : []),
    play: (p, i) => hooks.onPlay(p, i),
    attack: (p, s, t) => hooks.onAttack(p, s, t),
    evolve: (p, s, ev) => hooks.onEvolve(p, s, ev),
    getEvoArmed: () => evoArmed,
    clearEvoArmed: () => {
      evoArmed = null;
    },
  });

  byId("startGameBtn")?.addEventListener("click", () => void startFromForm());
  byId("undoBtn")?.addEventListener("click", () => applyHistory(undo));
  byId("redoBtn")?.addEventListener("click", () => applyHistory(redo));
  exposeArena();
  byId("modeSelect")?.addEventListener("change", syncModeChrome);
  byId("activeOnBottomToggle")?.addEventListener("change", (e) => {
    document.body.classList.toggle(
      "active-on-bottom",
      (e.target as HTMLInputElement).checked,
    );
  });
  byId("copySeedBtn")?.addEventListener("click", async () => {
    const v = byId("gameSeedValue")?.textContent ?? "";
    try {
      await navigator.clipboard.writeText(v);
    } catch {
      toast(v);
    }
  });
  byId("vsBotPolicy")?.addEventListener("change", () => {
    const vs = byId<HTMLSelectElement>("vsBotPolicy");
    const a = byId<HTMLSelectElement>("policyASelect");
    const b = byId<HTMLSelectElement>("policyBSelect");
    if (!vs || !a || !b) return;
    if (humanSideFromForm() === "a") b.value = vs.value;
    else a.value = vs.value;
  });

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
