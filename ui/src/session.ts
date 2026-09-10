import { Game } from "../pkg/arena_wasm.js";
import type {
  EngineEvent,
  FullState,
  NeutralAction,
  PlayerId,
  PositionLog,
  SessionConfig,
} from "./types.ts";

export type Session = {
  cfg: SessionConfig;
  game: Game;
  past: Game[];
  future: Game[];
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
  s.past.forEach((g) => g.free());
  s.future.forEach((g) => g.free());
  if (s.checkpoint) s.checkpoint.game.free();
}

export function fullState(s: Session): FullState {
  return JSON.parse(s.game.full()) as FullState;
}

export function legalActions(s: Session): NeutralAction[] {
  return JSON.parse(s.game.legal()) as NeutralAction[];
}

export function applyAction(s: Session, action: NeutralAction): EngineEvent[] {
  const snap = s.game.clone();
  try {
    const raw = s.game.apply(JSON.stringify(action));
    const events = JSON.parse(raw) as EngineEvent[];
    s.past.push(snap);
    s.future.forEach((g) => g.free());
    s.future = [];
    s.actions.push(action);
    ingestEvents(s, events);
    return events;
  } catch (err) {
    snap.free();
    throw err;
  }
}

function ingestEvents(s: Session, events: EngineEvent[]): void {
  for (const ev of events) {
    s.events.push(ev);
    if ("play" in ev) {
      const p = ev.play as { player: PlayerId; card: string };
      s.played[p.player].push(p.card);
    }
    if ("destroy" in ev) {
      const d = ev.destroy as { card: string };
      const who = actingFromEvent(s, ev);
      s.destroyed[who].push(d.card);
    }
    if ("turn_start" in ev) s.ply += 1;
  }
}

function actingFromEvent(s: Session, ev: EngineEvent): PlayerId {
  const dest = ev.destroy as { slot?: number } | undefined;
  if (dest && typeof dest.slot === "number") {
    // Best-effort: last active from the event stream, else current active.
  }
  return (s.game.active() as PlayerId) ?? "a";
}

export function undo(s: Session): boolean {
  if (!s.past.length) return false;
  s.future.push(s.game);
  s.game = s.past.pop()!;
  s.actions.pop();
  s.suppressFloater = true;
  rebuildDerived(s);
  return true;
}

export function redo(s: Session): boolean {
  if (!s.future.length) return false;
  s.past.push(s.game);
  s.game = s.future.pop()!;
  s.suppressFloater = true;
  rebuildDerived(s);
  return true;
}

function rebuildDerived(s: Session): void {
  const replay = createSession(s.cfg);
  s.played = { a: [], b: [] };
  s.destroyed = { a: [], b: [] };
  s.events = [];
  s.ply = 0;
  const keep = s.actions.slice();
  for (const act of keep) {
    const events = JSON.parse(replay.game.apply(JSON.stringify(act))) as EngineEvent[];
    ingestEvents(replay, events);
  }
  s.events = replay.events;
  s.played = replay.played;
  s.destroyed = replay.destroyed;
  s.ply = replay.ply;
  replay.game.free();
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
  s.past.forEach((g) => g.free());
  s.future.forEach((g) => g.free());
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
  return applyAction(s, action);
}
