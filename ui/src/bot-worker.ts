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
};

export type DecideReply =
  | { type: "decided"; id: number; action: NeutralAction; ms: number }
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

function cleanStep(step: LogStep): LogStep {
  if (isReseedStep(step)) return step;
  const raw = JSON.parse(JSON.stringify(step)) as Record<string, unknown>;
  delete raw.value;
  delete raw.bot_value;
  return raw as LogStep;
}

function replayActions(
  game: Game,
  actions: LogStep[],
): void {
  for (const step of actions) {
    if (isReseedStep(step)) {
      game.reseed(step.reseed);
    } else {
      game.apply(JSON.stringify(cleanStep(step)));
    }
  }
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

self.onmessage = (ev: MessageEvent<DecideRequest | BenchRequest>) => {
  void ensureWasm().then(() => {
    const msg = ev.data;
    if (msg.type === "decide") {
      const reply = handleDecide(msg);
      self.postMessage(reply);
    } else if (msg.type === "bench") {
      const reply = handleBench(msg);
      self.postMessage(reply);
    }
  });
};
