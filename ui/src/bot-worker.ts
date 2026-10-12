/// <reference lib="webworker" />
import init, { Game, version } from "../pkg/arena_wasm.js";
import type { LogStep, NeutralAction } from "./types.ts";
import { isReseedStep } from "./types.ts";

let wasmReady: Promise<void> | null = null;

function ensureWasm(): Promise<void> {
  if (!wasmReady) wasmReady = init().then(() => undefined);
  return wasmReady;
}

export type DecideRequest = {
  type: "decide";
  id: number;
  seed: string;
  deckA: string;
  deckB: string;
  first: string;
  actions: LogStep[];
  policy: string;
  botSeed: string;
  hash: string;
  fail?: boolean;
  /** Test-only: busy-spin in the worker before `botAction` (`?botworkerspin=`). */
  spinMs?: number;
};

export type DecideReply =
  | { type: "decided"; id: number; action: NeutralAction; ms: number }
  | { type: "error"; id: number; message: string };

export type PrepareRequest = {
  type: "prepare";
  id: number;
  seed: string;
  deckA: string;
  deckB: string;
  first: string;
  actions: LogStep[];
  hash: string;
  policy: string;
  botSeed: string;
  /** Test sabotage: wrong seed on world workers. */
  worldSeedBump?: number;
};

export type PoolJobRequest =
  | { type: "h0JobStart"; id: number }
  | { type: "h0KillDet"; id: number }
  | { type: "h0KillRoll"; id: number }
  | { type: "h0JobKillDet"; id: number; outcome: string }
  | { type: "h0JobKillRoll"; id: number; outcome: string }
  | {
      type: "h0JobWorld";
      id: number;
      r: number;
      budget: number;
    }
  | { type: "h0JobFinish"; id: number; outcomes: string }
  | { type: "release"; id: number };

export type PrepareReply =
  | { type: "prepared"; id: number }
  | { type: "error"; id: number; message: string };

export type PoolJobReply =
  | { type: "step"; id: number; stepJson: string }
  | { type: "outcome"; id: number; outcomeJson: string }
  | { type: "decided"; id: number; action: NeutralAction; ms: number }
  | { type: "released"; id: number }
  | { type: "error"; id: number; message: string };

export type BenchPosition = {
  i: number;
  deckAId: string;
  deckBId: string;
  deckA: string;
  deckB: string;
  seed: number;
  targetTurn: number;
};

export type BenchRequest = {
  type: "bench";
  id: number;
  spec: string;
  drivePolicy: string;
  positions: BenchPosition[];
};

export type BenchLine = {
  i: number;
  deckA: string;
  deckB: string;
  turn: number | "ended";
  ms: number | null;
  actionJson: string | "ended";
};

export type BenchReply =
  | { type: "bench-progress"; id: number; current: number; total: number }
  | { type: "bench-done"; id: number; lines: BenchLine[]; version: string }
  | { type: "bench-error"; id: number; message: string };

export type WorkerInbound =
  | DecideRequest
  | PrepareRequest
  | PoolJobRequest
  | BenchRequest;

export type WorkerOutbound =
  | DecideReply
  | PrepareReply
  | PoolJobReply
  | BenchReply;

function cleanStep(step: LogStep): LogStep {
  if (isReseedStep(step)) return step;
  const raw = JSON.parse(JSON.stringify(step)) as Record<string, unknown>;
  delete raw.value;
  delete raw.bot_value;
  return raw as LogStep;
}

function replayActions(game: Game, actions: LogStep[]): void {
  for (const step of actions) {
    if (isReseedStep(step)) {
      game.reseed(step.reseed);
    } else {
      game.apply(JSON.stringify(cleanStep(step)));
    }
  }
}

let heldGame: Game | null = null;
let heldPolicy = "";
let heldBotSeed = "";
let heldWorldSeedBump = 0;

function releaseHeld(): void {
  if (heldGame) {
    heldGame.free();
    heldGame = null;
  }
  heldPolicy = "";
  heldBotSeed = "";
  heldWorldSeedBump = 0;
}

function worldSeed(): string {
  const base = BigInt(heldBotSeed);
  return (base + BigInt(heldWorldSeedBump)).toString();
}

function reachPosition(
  seed: number,
  deckA: string,
  deckB: string,
  targetTurn: number,
  drivePolicy: string,
): { game: Game; n: number; ended: boolean } {
  const game = new Game(String(seed), deckA, deckB, "a");
  let n = 0;
  for (let step = 0; step < 400; step++) {
    if (game.phase() === "terminal") {
      return { game, n, ended: true };
    }
    const turn = game.turn();
    if (turn >= targetTurn && game.phase() === "main") {
      const legal = JSON.parse(game.legal()) as unknown[];
      if (legal.length >= 3) {
        return { game, n, ended: false };
      }
    }
    const raw = game.botAction(drivePolicy, String(seed + n));
    game.apply(raw);
    n += 1;
  }
  return { game, n, ended: true };
}

function busySpin(ms: number): void {
  if (ms <= 0) return;
  const end = performance.now() + ms;
  let x = 1;
  while (performance.now() < end) {
    x = (x * 1.000001 + 0.1) % 1e9;
  }
}

function handleDecide(req: DecideRequest): DecideReply {
  if (req.fail) {
    return { type: "error", id: req.id, message: "forced worker failure (test)" };
  }
  const game = new Game(req.seed, req.deckA, req.deckB, req.first);
  try {
    replayActions(game, req.actions);
    const h = game.hash();
    if (h !== req.hash) {
      return { type: "error", id: req.id, message: `hash mismatch (expected ${req.hash}, got ${h})` };
    }
    busySpin(req.spinMs ?? 0);
    const t0 = performance.now();
    const raw = game.botAction(req.policy, req.botSeed);
    const ms = performance.now() - t0;
    const action = JSON.parse(raw) as NeutralAction;
    return { type: "decided", id: req.id, action, ms };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { type: "error", id: req.id, message };
  } finally {
    game.free();
  }
}

function handlePrepare(req: PrepareRequest): PrepareReply {
  releaseHeld();
  try {
    const game = new Game(req.seed, req.deckA, req.deckB, req.first);
    replayActions(game, req.actions);
    const h = game.hash();
    if (h !== req.hash) {
      game.free();
      return { type: "error", id: req.id, message: `hash mismatch (expected ${req.hash}, got ${h})` };
    }
    heldGame = game;
    heldPolicy = req.policy;
    heldBotSeed = req.botSeed;
    heldWorldSeedBump = req.worldSeedBump ?? 0;
    return { type: "prepared", id: req.id };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { type: "error", id: req.id, message };
  }
}

function requireHeld(): Game {
  if (!heldGame) throw new Error("no prepared game");
  return heldGame;
}

function handlePoolJob(req: PoolJobRequest): PoolJobReply {
  try {
    if (req.type === "release") {
      releaseHeld();
      return { type: "released", id: req.id };
    }
    const game = requireHeld();
    if (req.type === "h0JobStart") {
      const t0 = performance.now();
      const stepJson = game.h0JobStart(heldPolicy, heldBotSeed);
      const step = JSON.parse(stepJson) as { step: string; action?: NeutralAction };
      const ms = performance.now() - t0;
      if (step.step === "done") {
        if (!step.action) {
          return { type: "error", id: req.id, message: `h0JobStart done without action: ${stepJson}` };
        }
        return { type: "decided", id: req.id, action: step.action, ms };
      }
      return { type: "step", id: req.id, stepJson };
    }
    if (req.type === "h0KillDet") {
      const outcomeJson = game.h0KillDet(heldPolicy, heldBotSeed);
      return { type: "outcome", id: req.id, outcomeJson };
    }
    if (req.type === "h0KillRoll") {
      const outcomeJson = game.h0KillRoll(heldPolicy, heldBotSeed);
      return { type: "outcome", id: req.id, outcomeJson };
    }
    if (req.type === "h0JobKillDet") {
      const stepJson = game.h0JobKillDet(req.outcome);
      return { type: "step", id: req.id, stepJson };
    }
    if (req.type === "h0JobKillRoll") {
      const stepJson = game.h0JobKillRoll(req.outcome);
      return { type: "step", id: req.id, stepJson };
    }
    if (req.type === "h0JobWorld") {
      const outcomeJson = game.h0JobWorld(heldPolicy, worldSeed(), req.r, req.budget);
      return { type: "outcome", id: req.id, outcomeJson };
    }
    if (req.type === "h0JobFinish") {
      const t0 = performance.now();
      const stepJson = game.h0JobFinish(req.outcomes);
      const step = JSON.parse(stepJson) as { step: string; action?: NeutralAction };
      const ms = performance.now() - t0;
      if (step.step !== "done" || !step.action) {
        return { type: "error", id: req.id, message: `h0JobFinish: ${stepJson}` };
      }
      return { type: "decided", id: req.id, action: step.action, ms };
    }
    return { type: "error", id: (req as { id: number }).id, message: "unknown pool job" };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { type: "error", id: req.id, message };
  }
}

function handleBench(req: BenchRequest): BenchReply {
  const lines: BenchLine[] = [];
  const total = req.positions.length;
  for (let idx = 0; idx < total; idx++) {
    const pos = req.positions[idx];
    self.postMessage({ type: "bench-progress", id: req.id, current: idx + 1, total });
    const { game, n, ended } = reachPosition(
      pos.seed,
      pos.deckA,
      pos.deckB,
      pos.targetTurn,
      req.drivePolicy,
    );
    try {
      if (ended || game.phase() === "terminal") {
        lines.push({
          i: pos.i,
          deckA: pos.deckAId,
          deckB: pos.deckBId,
          turn: "ended",
          ms: null,
          actionJson: "ended",
        });
      } else {
        const t0 = performance.now();
        const raw = game.botAction(req.spec, String(pos.seed + n));
        const ms = performance.now() - t0;
        lines.push({
          i: pos.i,
          deckA: pos.deckAId,
          deckB: pos.deckBId,
          turn: game.turn(),
          ms,
          actionJson: raw,
        });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      return { type: "bench-error", id: req.id, message };
    } finally {
      game.free();
    }
  }
  return { type: "bench-done", id: req.id, lines, version: version() };
}

self.onmessage = (ev: MessageEvent<WorkerInbound>) => {
  void ensureWasm().then(() => {
    const msg = ev.data;
    if (msg.type === "decide") {
      const reply = handleDecide(msg);
      self.postMessage(reply);
    } else if (msg.type === "prepare") {
      const reply = handlePrepare(msg);
      self.postMessage(reply);
    } else if (
      msg.type === "h0JobStart" ||
      msg.type === "h0KillDet" ||
      msg.type === "h0KillRoll" ||
      msg.type === "h0JobKillDet" ||
      msg.type === "h0JobKillRoll" ||
      msg.type === "h0JobWorld" ||
      msg.type === "h0JobFinish" ||
      msg.type === "release"
    ) {
      const reply = handlePoolJob(msg);
      self.postMessage(reply);
    } else if (msg.type === "bench") {
      const reply = handleBench(msg);
      self.postMessage(reply);
    }
  });
};
