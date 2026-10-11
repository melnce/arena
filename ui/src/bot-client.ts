import type {
  DecideReply,
  DecideRequest,
  PoolJobReply,
  PrepareReply,
  PrepareRequest,
  WorkerOutbound,
} from "./bot-worker.ts";
import {
  applyAction,
  botPolicyFor,
  botStepWithSeed,
  localBotRequestBody,
  nextBotSeed,
  type Session,
} from "./session.ts";
import type { EngineEvent, PlayerId } from "./types.ts";

export type BotWorkerStepMeta = { stale?: boolean; error?: string };

export type BenchDecideBody = {
  seed: string;
  deckA: string;
  deckB: string;
  first: string;
  actions: import("./types.ts").LogStep[];
  hash: string;
};

let lastMeta: BotWorkerStepMeta = {};
let workers: Worker[] = [];
let workerGeneration = 0;
let pendingId: number | null = null;
let nextRequestId = 1;
let permanentError: string | null = null;
let thinking = false;
let poolBusy = 0;

const BOT_CORES_STORAGE = "svwb.botCores";

function notifyThinkingChange(): void {
  window.dispatchEvent(new Event("arena-bot-backend"));
}

export function lastBotWorkerStepMeta(): BotWorkerStepMeta {
  return lastMeta;
}

export function isBotWorkerThinking(): boolean {
  return thinking;
}

function botWorkerQueryFlag(): string | null {
  try {
    return new URLSearchParams(location.search).get("botworker");
  } catch {
    return null;
  }
}

function botCoresQueryOverride(): number | null {
  try {
    const raw = new URLSearchParams(location.search).get("botcores");
    if (!raw) return null;
    const n = Number(raw);
    if (!Number.isFinite(n) || n < 1) return null;
    return Math.min(8, Math.floor(n));
  } catch {
    return null;
  }
}

export function autoBotCores(): number {
  const hw = navigator.hardwareConcurrency ?? 1;
  return Math.max(1, Math.min(8, hw - 1));
}

export function configuredBotCores(): number {
  const q = botCoresQueryOverride();
  if (q != null) return q;
  try {
    const stored = localStorage.getItem(BOT_CORES_STORAGE);
    if (stored && stored !== "auto") {
      const n = Number(stored);
      if (Number.isFinite(n) && n >= 1) return Math.min(8, Math.floor(n));
    }
  } catch {
    /* ignore */
  }
  return autoBotCores();
}

export function persistBotCoresChoice(value: string): void {
  try {
    localStorage.setItem(BOT_CORES_STORAGE, value);
  } catch {
    /* ignore */
  }
}

function botWorkerSpinMs(): number {
  try {
    const raw = new URLSearchParams(location.search).get("botworkerspin");
    if (!raw) return 0;
    const n = Number(raw);
    if (!Number.isFinite(n) || n <= 0) return 0;
    return Math.floor(n);
  } catch {
    return 0;
  }
}

function poolSabotageWorldSeed(): boolean {
  try {
    return new URLSearchParams(location.search).get("botpoolsabotage") === "worldseed";
  } catch {
    return false;
  }
}

export function poolSkipStaleTerminate(): boolean {
  try {
    return new URLSearchParams(location.search).get("botpoolsabotage") === "nostale";
  } catch {
    return false;
  }
}

export function botWorkerEnabled(): boolean {
  if (botWorkerQueryFlag() === "0") return false;
  if (permanentError) return false;
  return true;
}

export function botWorkerState(): {
  enabled: boolean;
  alive: boolean;
  pending: boolean;
  generation: number;
  error: string | null;
} {
  return {
    enabled: botWorkerEnabled(),
    alive: workers.some((w) => w != null),
    pending: pendingId != null,
    generation: workerGeneration,
    error: permanentError,
  };
}

export function botPoolState(): {
  size: number;
  alive: number;
  busy: number;
  generation: number;
} {
  const alive = workers.filter((w) => w != null).length;
  return {
    size: configuredBotCores(),
    alive,
    busy: poolBusy,
    generation: workerGeneration,
  };
}

function spawnWorkerAt(index: number): Worker {
  const w = new Worker(new URL("./bot-worker.ts", import.meta.url), { type: "module" });
  workers[index] = w;
  return w;
}

function ensureWorkerPool(coreCount?: number): void {
  const want = coreCount ?? configuredBotCores();
  while (workers.length < want) {
    spawnWorkerAt(workers.length);
  }
  for (let i = 0; i < want; i++) {
    if (!workers[i]) spawnWorkerAt(i);
  }
}

export function prewarmBotWorker(): void {
  if (!botWorkerEnabled()) return;
  ensureWorkerPool();
}

export function terminateBotWorker(): void {
  for (const w of workers) {
    w?.terminate();
  }
  workers = [];
  workerGeneration += 1;
  pendingId = null;
  thinking = false;
  poolBusy = 0;
  notifyThinkingChange();
}

function policyUsesWorker(policy: string): boolean {
  return policy === "h0" || policy.startsWith("h0:");
}

export function shouldUseBotWorker(s: Session): boolean {
  if (!botWorkerEnabled()) return false;
  if (s.game.phase() === "mulligan") return false;
  const policy = botPolicyFor(s, s.game.acting() as PlayerId);
  return policyUsesWorker(policy);
}

function postAndWait<T extends WorkerOutbound>(
  worker: Worker,
  msg: object,
  expectId: number,
  match: (data: WorkerOutbound) => T | null,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const onMessage = (ev: MessageEvent<WorkerOutbound>) => {
      const data = ev.data;
      if (data.id !== expectId) return;
      const hit = match(data);
      if (!hit) return;
      worker.removeEventListener("message", onMessage);
      worker.removeEventListener("error", onError);
      if (hit.type === "error") {
        reject(new Error(hit.message));
      } else {
        resolve(hit);
      }
    };
    const onError = () => {
      worker.removeEventListener("message", onMessage);
      worker.removeEventListener("error", onError);
      reject(new Error("worker crashed"));
    };
    worker.addEventListener("message", onMessage);
    worker.addEventListener("error", onError);
    worker.postMessage(msg);
  });
}

async function prepareWorker(
  workerIndex: number,
  id: number,
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  worldSeedBump: number,
): Promise<void> {
  const w = workers[workerIndex]!;
  const req: PrepareRequest = {
    type: "prepare",
    id,
    seed: body.seed,
    deckA: body.deckA,
    deckB: body.deckB,
    first: body.first,
    actions: body.actions,
    hash: body.hash,
    policy,
    botSeed,
    worldSeedBump,
  };
  poolBusy += 1;
  try {
    await postAndWait<PrepareReply>(w, req, id, (data) => {
      if (data.type === "prepared" || data.type === "error") return data as PrepareReply;
      return null;
    });
  } finally {
    poolBusy -= 1;
  }
}

async function poolJob<T extends PoolJobReply>(
  workerIndex: number,
  id: number,
  msg: object,
  match: (data: WorkerOutbound) => T | null,
): Promise<T> {
  const w = workers[workerIndex]!;
  poolBusy += 1;
  try {
    return await postAndWait<T>(w, msg, id, match);
  } finally {
    poolBusy -= 1;
  }
}

async function releaseWorker(workerIndex: number, id: number): Promise<void> {
  const w = workers[workerIndex];
  if (!w) return;
  try {
    await postAndWait(w, { type: "release", id }, id, (data) => {
      if (data.type === "released" || data.type === "error") return data;
      return null;
    });
  } catch {
    /* ignore release errors */
  }
}

function respawnWorker(index: number): void {
  workers[index]?.terminate();
  spawnWorkerAt(index);
  workerGeneration += 1;
}

async function runPoolDecide(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  spinMs: number,
  coreCount?: number,
): Promise<NeutralActionHolder> {
  const W = coreCount ?? configuredBotCores();
  ensureWorkerPool(W);
  const sabotage = poolSabotageWorldSeed();

  await Promise.all(
    Array.from({ length: W }, (_, i) =>
      prepareWorker(i, nextRequestId++, body, policy, botSeed, sabotage && i > 0 ? 1 : 0),
    ),
  );

  if (spinMs > 0) {
    const end = performance.now() + spinMs;
    while (performance.now() < end) {
      /* main-thread spin for cancel tests */
    }
  }

  const t0 = performance.now();
  const startMsgId = nextRequestId++;
  let start = await poolJob(
    0,
    startMsgId,
    { type: "h0JobStart", id: startMsgId },
    (data) => {
      if (data.type === "step" || data.type === "decided" || data.type === "error") return data;
      return null;
    },
  );

  while (start.type === "step") {
    const step = JSON.parse(start.stepJson) as {
      step: string;
      roll?: boolean;
      budgets?: number[];
    };
    if (step.step === "kill") {
      const detMsgId = nextRequestId++;
      const detP = poolJob(0, detMsgId, { type: "h0KillDet", id: detMsgId }, (data) => {
        if (data.type === "outcome" || data.type === "error") return data;
        return null;
      });
      let rollOutcome: string | null = null;
      let rollP: Promise<PoolJobReply> | null = null;
      if (step.roll && W > 1) {
        const rollMsgId = nextRequestId++;
        rollP = poolJob(1, rollMsgId, { type: "h0KillRoll", id: rollMsgId }, (data) => {
          if (data.type === "outcome" || data.type === "error") return data;
          return null;
        });
        rollP.then((roll) => {
          if (roll.type === "outcome") rollOutcome = roll.outcomeJson;
        });
      }
      const det = await detP;
      if (det.type !== "outcome") throw new Error("kill det failed");
      const feedDetId = nextRequestId++;
      start = await poolJob(
        0,
        feedDetId,
        { type: "h0JobKillDet", id: feedDetId, outcome: det.outcomeJson },
        (data) => {
          if (data.type === "step" || data.type === "decided" || data.type === "error") return data;
          return null;
        },
      );
      const needsRoll =
        start.type === "step" && (JSON.parse(start.stepJson) as { step: string }).step === "await_roll";
      if (!needsRoll) {
        if (step.roll && W > 1) respawnWorker(1);
      } else if (step.roll) {
        if (!rollOutcome) {
          if (rollP) {
            const roll = await rollP;
            if (roll.type !== "outcome") throw new Error("kill roll failed");
            rollOutcome = roll.outcomeJson;
          } else {
            const rollMsgId = nextRequestId++;
            const roll = await poolJob(
              0,
              rollMsgId,
              { type: "h0KillRoll", id: rollMsgId },
              (data) => {
                if (data.type === "outcome" || data.type === "error") return data;
                return null;
              },
            );
            if (roll.type !== "outcome") throw new Error("kill roll failed");
            rollOutcome = roll.outcomeJson;
          }
        }
        const feedRollId = nextRequestId++;
        start = await poolJob(
          0,
          feedRollId,
          { type: "h0JobKillRoll", id: feedRollId, outcome: rollOutcome },
          (data) => {
            if (data.type === "step" || data.type === "decided" || data.type === "error") return data;
            return null;
          },
        );
        if (W > 1) respawnWorker(1);
      }
      continue;
    }
    if (step.step === "await_roll") {
      const rollMsgId = nextRequestId++;
      const roll = await poolJob(
        0,
        rollMsgId,
        { type: "h0KillRoll", id: rollMsgId },
        (data) => {
          if (data.type === "outcome" || data.type === "error") return data;
          return null;
        },
      );
      if (roll.type !== "outcome") throw new Error("await_roll failed");
      const feedRollId = nextRequestId++;
      start = await poolJob(
        0,
        feedRollId,
        { type: "h0JobKillRoll", id: feedRollId, outcome: roll.outcomeJson },
        (data) => {
          if (data.type === "step" || data.type === "decided" || data.type === "error") return data;
          return null;
        },
      );
      continue;
    }
    if (step.step === "worlds" && step.budgets) {
      const budgets = step.budgets;
      const k = budgets.length;
      const outcomes: string[] = new Array(k);
      let nextR = 0;
      await Promise.all(
        Array.from({ length: W }, async (_, wi) => {
          while (true) {
            const r = nextR++;
            if (r >= k) break;
            const msgId = nextRequestId++;
            const out = await poolJob(
              wi,
              msgId,
              { type: "h0JobWorld", id: msgId, r, budget: budgets[r]! },
              (data) => {
                if (data.type === "outcome" || data.type === "error") return data;
                return null;
              },
            );
            if (out.type !== "outcome") throw new Error("world failed");
            outcomes[r] = out.outcomeJson;
          }
        }),
      );
      const finishId = nextRequestId++;
      start = await poolJob(
        0,
        finishId,
        { type: "h0JobFinish", id: finishId, outcomes: JSON.stringify(outcomes) },
        (data) => {
          if (data.type === "decided" || data.type === "error") return data;
          return null;
        },
      );
      continue;
    }
    throw new Error(`unexpected job step ${start.stepJson}`);
  }

  if (start.type !== "decided") throw new Error("pool decide failed");
  const ms = performance.now() - t0;
  await Promise.all(
    Array.from({ length: W }, (_, i) => releaseWorker(i, nextRequestId++)),
  );
  return { action: start.action, ms };
}

type NeutralActionHolder = { action: import("./types.ts").NeutralAction; ms: number };

async function runSingleWorkerDecide(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  id: number,
  fail: boolean,
  spinMs: number,
): Promise<NeutralActionHolder> {
  ensureWorkerPool();
  const w = workers[0]!;
  const req: DecideRequest = {
    type: "decide",
    id,
    ...body,
    policy,
    botSeed,
    spinMs,
    fail,
  };
  const reply = await postAndWait<DecideReply>(w, req, id, (data) => {
    if (data.type === "decided" || data.type === "error") return data as DecideReply;
    return null;
  });
  if (reply.type === "error") throw new Error(reply.message);
  return { action: reply.action, ms: reply.ms };
}

export async function botStepWorker(
  s: Session,
  opts?: { isCurrent?: () => boolean },
): Promise<EngineEvent[]> {
  const seed = nextBotSeed(s);
  const acting = s.game.acting() as PlayerId;
  const policy = botPolicyFor(s, acting);
  lastMeta = {};

  const W = configuredBotCores();
  ensureWorkerPool();
  const id = (pendingId = nextRequestId++);
  thinking = true;
  notifyThinkingChange();

  const sentHash = s.game.hash();
  const sentActions = s.actions.length;
  const sentBotSeq = s.botSeq;

  const dropStale = (): boolean => {
    if (opts?.isCurrent && !opts.isCurrent()) {
      lastMeta = { stale: true };
      return true;
    }
    try {
      if (
        s.game.hash() !== sentHash ||
        s.actions.length !== sentActions ||
        s.botSeq !== sentBotSeq
      ) {
        lastMeta = { stale: true };
        return true;
      }
    } catch {
      lastMeta = { stale: true };
      return true;
    }
    return false;
  };

  const body = toBenchBody(localBotRequestBody(s, policy, seed.toString()));
  const spinMs = botWorkerSpinMs();
  const fail = botWorkerQueryFlag() === "fail";

  return new Promise<EngineEvent[]>((resolve) => {
    let settled = false;
    let pollTimer: number | null = null;

    const finish = (events: EngineEvent[]) => {
      if (settled) return;
      settled = true;
      pendingId = null;
      thinking = false;
      notifyThinkingChange();
      if (pollTimer != null) window.clearInterval(pollTimer);
      resolve(events);
    };

    const onStale = () => {
      if (!poolSkipStaleTerminate()) terminateBotWorker();
      finish([]);
    };

    pollTimer = window.setInterval(() => {
      if (dropStale()) onStale();
    }, 200);

    void (async () => {
      try {
        const decided =
          W <= 1
            ? await runSingleWorkerDecide(body, policy, seed.toString(), id, fail, spinMs)
            : await runPoolDecide(body, policy, seed.toString(), spinMs);
        if (dropStale()) {
          onStale();
          return;
        }
        const events = applyAction(s, decided.action, { human: false });
        finish(events);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        permanentError = permanentError ?? message;
        terminateBotWorker();
        if (dropStale()) {
          finish([]);
          return;
        }
        lastMeta = { error: message };
        finish(botStepWithSeed(s, seed));
      }
    })();
  });
}

/** Bench helper: run one bot decision at a replayed position (pool or single worker). */
function toBenchBody(body: ReturnType<typeof localBotRequestBody>): BenchDecideBody {
  const { policy: _p, botSeed: _b, ...rest } = body;
  return rest;
}

export async function benchDecideAtPosition(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  cores: number,
): Promise<{ actionJson: string; ms: number }> {
  if (cores <= 1) {
    ensureWorkerPool(1);
    const id = nextRequestId++;
    const decided = await runSingleWorkerDecide(body, policy, botSeed, id, false, 0);
    return { actionJson: JSON.stringify(decided.action), ms: decided.ms };
  }
  const decided = await runPoolDecide(body, policy, botSeed, 0, cores);
  return { actionJson: JSON.stringify(decided.action), ms: decided.ms };
}
