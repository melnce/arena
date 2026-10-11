import type { DecideReply, DecideRequest } from "./bot-worker.ts";
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

let lastMeta: BotWorkerStepMeta = {};
let worker: Worker | null = null;
let workerGeneration = 0;
let pendingId: number | null = null;
let permanentError: string | null = null;
let thinking = false;

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
    alive: worker != null,
    pending: pendingId != null,
    generation: workerGeneration,
    error: permanentError,
  };
}

function spawnWorker(): Worker {
  workerGeneration += 1;
  const w = new Worker(new URL("./bot-worker.ts", import.meta.url), { type: "module" });
  worker = w;
  return w;
}

export function prewarmBotWorker(): void {
  if (!botWorkerEnabled()) return;
  if (!worker) spawnWorker();
}

export function terminateBotWorker(): void {
  if (worker) {
    worker.terminate();
    worker = null;
  }
  pendingId = null;
  thinking = false;
  notifyThinkingChange();
}

function policyUsesWorker(policy: string): boolean {
  return policy === "h0" || policy.startsWith("h0:");
}

export function shouldUseBotWorker(s: Session): boolean {
  if (!botWorkerEnabled()) return false;
  const policy = botPolicyFor(s, s.game.acting() as PlayerId);
  return policyUsesWorker(policy);
}

export async function botStepWorker(
  s: Session,
  opts?: { isCurrent?: () => boolean },
): Promise<EngineEvent[]> {
  const seed = nextBotSeed(s);
  const acting = s.game.acting() as PlayerId;
  const policy = botPolicyFor(s, acting);
  lastMeta = {};

  if (botWorkerQueryFlag() === "fail") {
    permanentError = permanentError ?? "forced worker failure (test)";
    terminateBotWorker();
    lastMeta = { error: permanentError };
    return botStepWithSeed(s, seed);
  }

  if (!worker) spawnWorker();
  const w = worker!;
  const id = (pendingId = Date.now());
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

  const body = localBotRequestBody(s, policy, seed.toString());
  const req: DecideRequest = { type: "decide", id, ...body };

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
      w.removeEventListener("message", onMessage);
      w.removeEventListener("error", onError);
      resolve(events);
    };

    const onStale = () => {
      terminateBotWorker();
      finish([]);
    };

    pollTimer = window.setInterval(() => {
      if (dropStale()) onStale();
    }, 200);

    const onError = () => {
      permanentError = permanentError ?? "worker crashed";
      terminateBotWorker();
      if (dropStale()) {
        finish([]);
        return;
      }
      lastMeta = { error: permanentError };
      finish(botStepWithSeed(s, seed));
    };

    const onMessage = (ev: MessageEvent<DecideReply>) => {
      const data = ev.data;
      if (data.id !== id) return;
      if (data.type === "error") {
        permanentError = data.message;
        terminateBotWorker();
        if (dropStale()) {
          finish([]);
          return;
        }
        lastMeta = { error: data.message };
        finish(botStepWithSeed(s, seed));
        return;
      }
      if (dropStale()) {
        onStale();
        return;
      }
      const events = applyAction(s, data.action, { human: false });
      finish(events);
    };

    w.addEventListener("message", onMessage);
    w.addEventListener("error", onError);
    w.postMessage(req);
  });
}
