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
  isHumanActing,
  localBotRequestBody,
  nextBotSeed,
  type Session,
} from "./session.ts";
import type { EngineEvent, PlayerId } from "./types.ts";

export type BotWorkerStepMeta = {
  stale?: boolean;
  error?: string;
  ms?: number;
  backend?: "worker" | "worker-retry" | "main";
};

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
let benchWorkers: Worker[] = [];
let benchNextRequestId = 1;
let workerGeneration = 0;
let pendingId: number | null = null;
let nextRequestId = 1;
let permanentError: string | null = null;
let consecutiveWorkerDecisionFailures = 0;
let failOnceArmed = false;
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

function botSlowMainMs(): number {
  try {
    const raw = new URLSearchParams(location.search).get("botslowmain");
    if (!raw) return 0;
    const n = Number(raw);
    if (!Number.isFinite(n) || n <= 0) return 0;
    return Math.min(10_000, Math.floor(n));
  } catch {
    return 0;
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
  if (consecutiveWorkerDecisionFailures >= 3) return false;
  return true;
}

/** Reset worker error badge and per-game fallback counter (new vs-bot session). */
export function resetBotWorkerGameState(): void {
  permanentError = null;
  consecutiveWorkerDecisionFailures = 0;
  failOnceArmed = botWorkerQueryFlag() === "failonce";
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

function spawnBenchWorkerAt(index: number): Worker {
  const w = new Worker(new URL("./bot-worker.ts", import.meta.url), { type: "module" });
  benchWorkers[index] = w;
  return w;
}

function ensureBenchWorkerPool(coreCount: number): void {
  const want = coreCount;
  while (benchWorkers.length < want) {
    spawnBenchWorkerAt(benchWorkers.length);
  }
  for (let i = 0; i < want; i++) {
    if (!benchWorkers[i]) spawnBenchWorkerAt(i);
  }
}

function ensureWorkerPool(coreCount?: number): void {
  const want = coreCount ?? configuredBotCores();
  const hadWorkers = workers.some((w) => w != null);
  while (workers.length < want) {
    spawnWorkerAt(workers.length);
  }
  for (let i = 0; i < want; i++) {
    if (!workers[i]) spawnWorkerAt(i);
  }
  if (!hadWorkers && want > 0) workerGeneration += 1;
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

export function policyUsesWorker(policy: string): boolean {
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

function trackPoolBusy(pool: Worker[]): { done: () => void } {
  if (pool !== workers) return { done: () => {} };
  poolBusy += 1;
  return { done: () => {
    poolBusy -= 1;
  } };
}

async function prepareWorker(
  pool: Worker[],
  workerIndex: number,
  id: number,
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  worldSeedBump: number,
): Promise<void> {
  const w = pool[workerIndex]!;
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
  const busy = trackPoolBusy(pool);
  try {
    await postAndWait<PrepareReply>(w, req, id, (data) => {
      if (data.type === "prepared" || data.type === "error") return data as PrepareReply;
      return null;
    });
  } finally {
    busy.done();
  }
}

async function poolJob<T extends PoolJobReply>(
  pool: Worker[],
  workerIndex: number,
  id: number,
  msg: object,
  match: (data: WorkerOutbound) => T | null,
): Promise<T> {
  const w = pool[workerIndex]!;
  const busy = trackPoolBusy(pool);
  try {
    return await postAndWait<T>(w, msg, id, match);
  } finally {
    busy.done();
  }
}

async function releaseWorker(pool: Worker[], workerIndex: number, id: number): Promise<void> {
  const w = pool[workerIndex];
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

function respawnWorkerInPool(pool: Worker[], index: number): void {
  pool[index]?.terminate();
  if (pool === workers) spawnWorkerAt(index);
  else spawnBenchWorkerAt(index);
}

async function respawnAndPrepare(
  pool: Worker[],
  allocId: () => number,
  index: number,
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  worldSeedBump: number,
): Promise<void> {
  respawnWorkerInPool(pool, index);
  await prepareWorker(pool, index, allocId(), body, policy, botSeed, worldSeedBump);
}

async function runPoolDecide(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  spinMs: number,
  coreCount?: number,
  opts?: { workerPool?: Worker[] },
): Promise<NeutralActionHolder> {
  const W = coreCount ?? configuredBotCores();
  const pool = opts?.workerPool ?? workers;
  const gamePool = pool === workers;
  const allocId = () => (gamePool ? nextRequestId++ : benchNextRequestId++);
  if (gamePool) ensureWorkerPool(W);
  else ensureBenchWorkerPool(W);
  const sabotage = poolSabotageWorldSeed();
  if (gamePool) poolBusy = W;
  try {
  if (spinMs > 0) {
    const end = performance.now() + spinMs;
    while (performance.now() < end) {
      await new Promise<void>((r) => window.setTimeout(r, 40));
    }
  }

  await Promise.all(
    Array.from({ length: W }, (_, i) =>
      prepareWorker(
        pool,
        i,
        allocId(),
        body,
        policy,
        botSeed,
        sabotage && i > 0 ? 1 : 0,
      ),
    ),
  );

  const t0 = performance.now();
  const startMsgId = allocId();
  let start = await poolJob(
    pool,
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
      const detMsgId = allocId();
      const detP = poolJob(pool, 0, detMsgId, { type: "h0KillDet", id: detMsgId }, (data) => {
        if (data.type === "outcome" || data.type === "error") return data;
        return null;
      });
      let rollOutcome: string | null = null;
      let rollP: Promise<PoolJobReply> | null = null;
      if (step.roll && W > 1) {
        const rollMsgId = allocId();
        rollP = poolJob(pool, 1, rollMsgId, { type: "h0KillRoll", id: rollMsgId }, (data) => {
          if (data.type === "outcome" || data.type === "error") return data;
          return null;
        });
        rollP.then((roll) => {
          if (roll.type === "outcome") rollOutcome = roll.outcomeJson;
        });
      }
      const det = await detP;
      if (det.type !== "outcome") throw new Error("kill det failed");
      const feedDetId = allocId();
      start = await poolJob(
        pool,
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
        if (step.roll && W > 1) await respawnAndPrepare(pool, allocId, 1, body, policy, botSeed, 0);
      } else if (step.roll) {
        if (!rollOutcome) {
          if (rollP) {
            const roll = await rollP;
            if (roll.type !== "outcome") throw new Error("kill roll failed");
            rollOutcome = roll.outcomeJson;
          } else {
            const rollMsgId = allocId();
            const roll = await poolJob(
              pool,
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
        const feedRollId = allocId();
        start = await poolJob(
          pool,
          0,
          feedRollId,
          { type: "h0JobKillRoll", id: feedRollId, outcome: rollOutcome },
          (data) => {
            if (data.type === "step" || data.type === "decided" || data.type === "error") return data;
            return null;
          },
        );
        if (W > 1) await respawnAndPrepare(pool, allocId, 1, body, policy, botSeed, 0);
      }
      continue;
    }
    if (step.step === "await_roll") {
      const rollMsgId = allocId();
      const roll = await poolJob(
        pool,
        0,
        rollMsgId,
        { type: "h0KillRoll", id: rollMsgId },
        (data) => {
          if (data.type === "outcome" || data.type === "error") return data;
          return null;
        },
      );
      if (roll.type !== "outcome") throw new Error("await_roll failed");
      const feedRollId = allocId();
      start = await poolJob(
        pool,
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
      const outcomes: unknown[] = new Array(k);
      let nextR = 0;
      const worldWorkers = Array.from({ length: W }, (_, j) => j);
      await Promise.all(
        worldWorkers.map(async (wi) => {
          while (true) {
            const r = nextR++;
            if (r >= k) break;
            const msgId = allocId();
            const out = await poolJob(
              pool,
              wi,
              msgId,
              { type: "h0JobWorld", id: msgId, r, budget: budgets[r]! },
              (data) => {
                if (data.type === "outcome" || data.type === "error") return data;
                return null;
              },
            );
            if (out.type !== "outcome") throw new Error("world failed");
            outcomes[r] = JSON.parse(out.outcomeJson);
          }
        }),
      );
      const finishId = allocId();
      start = await poolJob(
        pool,
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
  if (!gamePool) {
    await Promise.all(
      Array.from({ length: W }, (_, i) => releaseWorker(pool, i, allocId())),
    );
  }
  return { action: start.action, ms };
  } finally {
    if (gamePool) poolBusy = 0;
  }
}

type NeutralActionHolder = { action: import("./types.ts").NeutralAction; ms: number };

async function runSingleWorkerDecide(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  id: number,
  fail: boolean,
  spinMs: number,
  workerPool?: Worker[],
): Promise<NeutralActionHolder> {
  const pool = workerPool ?? workers;
  if (pool === workers) ensureWorkerPool();
  else ensureBenchWorkerPool(1);
  const w = pool[0]!;
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

function showBotBusyOverlay(): void {
  let el = document.getElementById("botBusyOverlay");
  if (!el) {
    el = document.createElement("div");
    el.id = "botBusyOverlay";
    el.innerHTML = `<div class="bot-busy-card">Bot is thinking — slow mode</div>`;
    document.body.appendChild(el);
  }
  el.style.display = "flex";
}

function hideBotBusyOverlay(): void {
  const el = document.getElementById("botBusyOverlay");
  if (el) el.style.display = "none";
}

async function paintBotBusyOverlay(): Promise<void> {
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  await new Promise<void>((resolve) => window.setTimeout(resolve, 0));
}

type MainFallbackOpts = { isCurrent?: () => boolean };

type MainFallbackSnap = {
  hash: string;
  actions: number;
  botSeq: number;
};

function snapshotMainFallback(s: Session): MainFallbackSnap {
  return {
    hash: s.game.hash(),
    actions: s.actions.length,
    botSeq: s.botSeq,
  };
}

function mainFallbackStale(s: Session, snap: MainFallbackSnap, opts?: MainFallbackOpts): boolean {
  if (opts?.isCurrent && !opts.isCurrent()) return true;
  try {
    if (
      s.game.hash() !== snap.hash ||
      s.actions.length !== snap.actions ||
      s.botSeq !== snap.botSeq
    ) {
      return true;
    }
  } catch {
    return true;
  }
  if (s.game.phase() === "terminal" || isHumanActing(s)) return true;
  return false;
}

export async function botStepMainFallback(
  s: Session,
  seed: bigint,
  opts?: MainFallbackOpts,
): Promise<EngineEvent[]> {
  lastMeta = {};
  const snap = snapshotMainFallback(s);
  if (mainFallbackStale(s, snap, opts)) {
    lastMeta = { stale: true };
    return [];
  }
  showBotBusyOverlay();
  await paintBotBusyOverlay();
  const slow = botSlowMainMs();
  if (slow > 0) await new Promise<void>((r) => window.setTimeout(r, slow));
  if (mainFallbackStale(s, snap, opts)) {
    hideBotBusyOverlay();
    lastMeta = { stale: true };
    return [];
  }
  try {
    return botStepWithSeed(s, seed);
  } finally {
    hideBotBusyOverlay();
  }
}

type WorkerDecideResult = {
  action: import("./types.ts").NeutralAction;
  ms: number;
  backend: "worker" | "worker-retry";
  error?: string;
};

function forcedWorkerFailure(): Error {
  return new Error("forced worker failure (test)");
}

async function workerDecideOnce(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  spinMs: number,
  W: number,
  requestId: number,
  alwaysFail: boolean,
  poolFailOnce: boolean,
): Promise<NeutralActionHolder> {
  if (alwaysFail) throw forcedWorkerFailure();
  if (poolFailOnce) {
    failOnceArmed = false;
    throw forcedWorkerFailure();
  }
  if (W <= 1) {
    return runSingleWorkerDecide(body, policy, botSeed, requestId, false, spinMs);
  }
  return runPoolDecide(body, policy, botSeed, spinMs);
}

async function decideInWorker(
  body: BenchDecideBody,
  policy: string,
  botSeed: string,
  spinMs: number,
): Promise<WorkerDecideResult> {
  const W = configuredBotCores();
  const alwaysFail = botWorkerQueryFlag() === "fail";
  let decisionError: string | undefined;

  const attempt = async (retry: boolean): Promise<WorkerDecideResult> => {
    ensureWorkerPool(W);
    const poolFailOnce = !retry && failOnceArmed && W > 1;
    const id = nextRequestId++;
    try {
      const decided = await workerDecideOnce(
        body,
        policy,
        botSeed,
        spinMs,
        W,
        id,
        alwaysFail,
        poolFailOnce,
      );
      return {
        action: decided.action,
        ms: decided.ms,
        backend: retry ? "worker-retry" : "worker",
        error: retry ? decisionError : undefined,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      decisionError = message;
      permanentError = message;
      terminateBotWorker();
      throw err;
    }
  };

  try {
    return await attempt(false);
  } catch {
    try {
      return await attempt(true);
    } catch {
      terminateBotWorker();
      ensureWorkerPool(1);
      const id = nextRequestId++;
      try {
        const decided = await runSingleWorkerDecide(body, policy, botSeed, id, alwaysFail, spinMs);
        return {
          action: decided.action,
          ms: decided.ms,
          backend: "worker-retry",
          error: decisionError,
        };
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        permanentError = message;
        terminateBotWorker();
        throw new Error(decisionError ?? message);
      }
    }
  }
}

export async function botStepWorker(
  s: Session,
  opts?: { isCurrent?: () => boolean },
): Promise<EngineEvent[]> {
  const seed = nextBotSeed(s);
  const acting = s.game.acting() as PlayerId;
  const policy = botPolicyFor(s, acting);
  lastMeta = {};

  ensureWorkerPool();
  pendingId = nextRequestId++;
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
        const decided = await decideInWorker(body, policy, seed.toString(), spinMs);
        if (dropStale()) {
          onStale();
          return;
        }
        consecutiveWorkerDecisionFailures = 0;
        lastMeta = {
          ms: typeof decided.ms === "number" && Number.isFinite(decided.ms) ? decided.ms : undefined,
          backend: decided.backend,
          error: decided.error,
        };
        const events = applyAction(s, decided.action, { human: false });
        finish(events);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        if (dropStale()) {
          onStale();
          return;
        }
        consecutiveWorkerDecisionFailures += 1;
        lastMeta = { error: message, backend: "main" };
        const events = await botStepMainFallback(s, seed, { isCurrent: opts?.isCurrent });
        if (dropStale() || lastMeta.stale) {
          onStale();
          return;
        }
        finish(events);
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
    const id = benchNextRequestId++;
    const decided = await runSingleWorkerDecide(body, policy, botSeed, id, false, 0, benchWorkers);
    return { actionJson: JSON.stringify(decided.action), ms: decided.ms };
  }
  const decided = await runPoolDecide(body, policy, botSeed, 0, cores, { workerPool: benchWorkers });
  return { actionJson: JSON.stringify(decided.action), ms: decided.ms };
}
