import { Game } from "../pkg/arena_wasm.js";
import type {
  EngineEvent,
  FullState,
  NeutralAction,
  PlayerId,
  PositionLog,
  SessionConfig,
} from "./types.ts";

/** Ring limit — same as the old practice tool. */
export const HISTORY_LIMIT = 200;

export type HistStep = {
  before: Game;
  action: NeutralAction;
  events: EngineEvent[];
  human: boolean;
  botSeqBefore: number;
};

export type FutureStep = {
  after: Game;
  action: NeutralAction;
  events: EngineEvent[];
  human: boolean;
  botSeqBefore: number;
};

export type Session = {
  cfg: SessionConfig;
  game: Game;
  past: HistStep[];
  future: FutureStep[];
  actions: NeutralAction[];
  events: EngineEvent[];
  played: { a: string[]; b: string[] };
  destroyed: { a: string[]; b: string[] };
  ply: number;
  botSeq: number;
  checkpoint: {
    game: Game;
    actions: NeutralAction[];
    events: EngineEvent[];
    played: { a: string[]; b: string[] };
    destroyed: { a: string[]; b: string[] };
    ply: number;
    botSeq: number;
  } | null;
  mulliganSwap: [boolean, boolean, boolean, boolean];
  suppressFloater: boolean;
};

function deckJson(deck: Record<string, number>): string {
  return JSON.stringify(deck);
}

export function createSession(cfg: SessionConfig): Session {
  const game = new Game(cfg.seed.toString(), deckJson(cfg.deckA), deckJson(cfg.deckB), cfg.first);
  return {
    cfg,
    game,
    past: [],
    future: [],
    actions: [],
    events: [],
    played: { a: [], b: [] },
    destroyed: { a: [], b: [] },
    ply: 0,
    botSeq: 0,
    checkpoint: null,
    mulliganSwap: [false, false, false, false],
    suppressFloater: false,
  };
}

export function disposeSession(s: Session | null): void {
  if (!s) return;
  s.game.free();
  s.past.forEach((p) => p.before.free());
  s.future.forEach((f) => f.after.free());
  if (s.checkpoint) s.checkpoint.game.free();
}

export function fullState(s: Session): FullState {
  return JSON.parse(s.game.full()) as FullState;
}

export function legalActions(s: Session): NeutralAction[] {
  return JSON.parse(s.game.legal()) as NeutralAction[];
}

export function canUndo(s: Session): boolean {
  return s.past.length > 0;
}

export function canRedo(s: Session): boolean {
  return s.future.length > 0;
}

function clearFuture(s: Session): void {
  for (const f of s.future) f.after.free();
  s.future = [];
}

function trimPast(s: Session): void {
  while (s.past.length > HISTORY_LIMIT) {
    const dropped = s.past.shift();
    dropped?.before.free();
    s.actions.shift();
  }
}

export function applyAction(
  s: Session,
  action: NeutralAction,
  opts?: { human?: boolean },
): EngineEvent[] {
  const human = opts?.human !== false;
  const botSeqBefore = s.botSeq;
  const before = s.game.clone();
  try {
    const raw = s.game.apply(JSON.stringify(action));
    const events = JSON.parse(raw) as EngineEvent[];
    s.past.push({ before, action, events, human, botSeqBefore });
    clearFuture(s);
    s.actions.push(action);
    trimPast(s);
    rebuildDerived(s);
    return events;
  } catch (err) {
    before.free();
    throw err;
  }
}

function ingestEvents(into: Session, events: EngineEvent[]): void {
  for (const ev of events) {
    into.events.push(ev);
    if ("play" in ev) {
      const p = ev.play as { player: PlayerId; card: string };
      into.played[p.player].push(p.card);
    }
    if ("destroy" in ev) {
      const d = ev.destroy as { card: string };
      const who = (into.game.active() as PlayerId) ?? "a";
      into.destroyed[who].push(d.card);
    }
    if ("turn_start" in ev) into.ply += 1;
  }
}

/** Rebuild played / destroyed / ply / event log from the applied action list. */
function rebuildDerived(s: Session): void {
  s.played = { a: [], b: [] };
  s.destroyed = { a: [], b: [] };
  s.events = [];
  s.ply = 0;
  for (const step of s.past) ingestEvents(s, step.events);
}

function undoOne(s: Session): boolean {
  const step = s.past.pop();
  if (!step) return false;
  s.future.push({
    after: s.game,
    action: step.action,
    events: step.events,
    human: step.human,
    botSeqBefore: step.botSeqBefore,
  });
  s.game = step.before;
  s.botSeq = step.botSeqBefore;
  s.actions.pop();
  s.suppressFloater = true;
  rebuildDerived(s);
  return true;
}

function redoOne(s: Session): boolean {
  const item = s.future.pop();
  if (!item) return false;
  s.past.push({
    before: s.game,
    action: item.action,
    events: item.events,
    human: item.human,
    botSeqBefore: item.botSeqBefore,
  });
  s.game = item.after;
  s.botSeq = item.human ? item.botSeqBefore : item.botSeqBefore + 1;
  s.actions.push(item.action);
  s.suppressFloater = true;
  rebuildDerived(s);
  return true;
}

/**
 * Undo. Hotseat / watch: one NeutralAction, except mid-Choice which jumps to
 * the state before the choice began. Vs-bot: one jump to the human's previous
 * decision (bot actions in between ride along on the redo stack).
 */
export function undo(s: Session): boolean {
  if (!s.past.length) return false;
  if (s.cfg.mode === "watch") return undoOne(s);
  if (s.game.phase() === "choice") {
    let moved = false;
    do {
      if (!undoOne(s)) break;
      moved = true;
    } while (s.game.phase() === "choice" && s.past.length);
    return moved;
  }
  if (s.cfg.mode === "vs-bot") return undoToHumanDecision(s);
  return undoOne(s);
}

function undoToHumanDecision(s: Session): boolean {
  const lastHuman = s.past.map((p) => p.human).lastIndexOf(true);
  if (lastHuman < 0) {
    let moved = false;
    while (s.past.length) {
      undoOne(s);
      moved = true;
    }
    return moved;
  }
  const n = s.past.length - lastHuman;
  let moved = false;
  for (let i = 0; i < n; i++) {
    if (!undoOne(s)) break;
    moved = true;
  }
  return moved;
}

/**
 * Redo. Hotseat / watch: one snapshot. Vs-bot: the human action plus the
 * bot replies that rode with the matching undo, replayed from stored
 * snapshots (no new `botAction` roll).
 */
export function redo(s: Session): boolean {
  if (!s.future.length) return false;
  if (s.cfg.mode === "watch") return redoOne(s);
  if (s.cfg.mode === "vs-bot") return redoToHumanDecision(s);
  return redoOne(s);
}

function redoToHumanDecision(s: Session): boolean {
  if (!redoOne(s)) return false;
  while (s.future.length && !s.future[s.future.length - 1]!.human) {
    if (!redoOne(s)) break;
  }
  return true;
}

export function setCheckpoint(s: Session): void {
  if (s.checkpoint) s.checkpoint.game.free();
  s.checkpoint = {
    game: s.game.clone(),
    actions: s.actions.slice(),
    events: s.events.slice(),
    played: { a: s.played.a.slice(), b: s.played.b.slice() },
    destroyed: { a: s.destroyed.a.slice(), b: s.destroyed.b.slice() },
    ply: s.ply,
    botSeq: s.botSeq,
  };
}

export function restoreCheckpoint(s: Session): boolean {
  if (!s.checkpoint) return false;
  s.past.forEach((p) => p.before.free());
  s.future.forEach((f) => f.after.free());
  s.past = [];
  s.future = [];
  s.game.free();
  s.game = s.checkpoint.game.clone();
  s.actions = s.checkpoint.actions.slice();
  s.events = s.checkpoint.events.slice();
  s.played = { a: s.checkpoint.played.a.slice(), b: s.checkpoint.played.b.slice() };
  s.destroyed = {
    a: s.checkpoint.destroyed.a.slice(),
    b: s.checkpoint.destroyed.b.slice(),
  };
  s.ply = s.checkpoint.ply;
  s.botSeq = s.checkpoint.botSeq;
  s.suppressFloater = true;
  return true;
}

export function toPositionLog(s: Session): PositionLog {
  return {
    v: 1,
    kind: "replay-log",
    seed: s.cfg.seed.toString(),
    deckA: s.cfg.deckA,
    deckB: s.cfg.deckB,
    deckAId: s.cfg.deckAId,
    deckBId: s.cfg.deckBId,
    first: s.cfg.first,
    actions: s.actions.slice(),
  };
}

export function replayPosition(log: PositionLog): Session {
  const s = createSession({
    seed: BigInt(log.seed),
    deckA: log.deckA,
    deckB: log.deckB,
    deckAId: log.deckAId,
    deckBId: log.deckBId,
    first: log.first,
    mode: "hotseat",
    humanSide: "a",
    hideBotHand: false,
    policyA: "random",
    policyB: "random",
  });
  for (const act of log.actions) applyAction(s, act);
  s.suppressFloater = true;
  return s;
}

export function isHumanActing(s: Session): boolean {
  const acting = s.game.acting() as PlayerId;
  if (s.cfg.mode === "hotseat") return true;
  if (s.cfg.mode === "watch") return false;
  return acting === s.cfg.humanSide;
}

export function botPolicyFor(s: Session, who: PlayerId): string {
  return who === "a" ? s.cfg.policyA : s.cfg.policyB;
}

export function nextBotSeed(s: Session): bigint {
  const seed = s.cfg.seed + BigInt(s.botSeq);
  s.botSeq += 1;
  return seed;
}

export function botStep(s: Session): EngineEvent[] {
  const policy = botPolicyFor(s, s.game.acting() as PlayerId);
  const action = JSON.parse(s.game.botAction(policy, nextBotSeed(s).toString())) as NeutralAction;
  return applyAction(s, action, { human: false });
}
