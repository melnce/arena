import { timingSafeEqual } from "node:crypto";

export type CaptureConfig = {
  captureKey: string;
  ghToken: string;
  repo: string;
  branch: string;
};

export type FetchFn = typeof fetch;

const MAX_BODY = 1_048_576;
const GAME_ID_RE = /^[0-9]+-[0-9a-f]{8}$/;
const SESSION_ID_RE = /^[0-9a-z]{10}$/;

export function canonicalDeckJson(deck: Record<string, number>): string {
  const keys = Object.keys(deck).sort();
  const parts = keys.map((k) => `${JSON.stringify(k)}:${JSON.stringify(deck[k])}`);
  return `{${parts.join(",")}}`;
}

function constantTimeKeyOk(provided: string | null, expected: string): boolean {
  if (!provided || !expected) return false;
  const a = Buffer.from(provided, "utf8");
  const b = Buffer.from(expected, "utf8");
  if (a.length !== b.length) return false;
  return timingSafeEqual(a, b);
}

function parseDecks(body: Record<string, unknown>): { deckA: Record<string, number>; deckB: Record<string, number> } | string {
  const deckA = body.deckA;
  const deckB = body.deckB;
  if (!deckA || typeof deckA !== "object" || Array.isArray(deckA)) return "deckA must be an object";
  if (!deckB || typeof deckB !== "object" || Array.isArray(deckB)) return "deckB must be an object";
  for (const [name, deck] of [["deckA", deckA], ["deckB", deckB]] as const) {
    for (const [k, v] of Object.entries(deck as Record<string, unknown>)) {
      if (typeof k !== "string" || typeof v !== "number" || !Number.isFinite(v) || v <= 0 || !Number.isInteger(v)) {
        return `${name} must map card ids to positive integer counts`;
      }
    }
  }
  return { deckA: deckA as Record<string, number>, deckB: deckB as Record<string, number> };
}

export function validateCaptureBody(raw: string): { ok: true; capture: Record<string, unknown> } | { ok: false; error: string } {
  if (raw.length > MAX_BODY) return { ok: false, error: "body too large" };
  let body: unknown;
  try {
    body = JSON.parse(raw);
  } catch {
    return { ok: false, error: "invalid JSON" };
  }
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    return { ok: false, error: "body must be an object" };
  }
  const rec = body as Record<string, unknown>;
  if (rec.v !== 1) return { ok: false, error: "v must be 1" };
  const gameId = rec.game_id;
  if (typeof gameId !== "string" || !GAME_ID_RE.test(gameId)) {
    return { ok: false, error: "game_id format invalid" };
  }
  const sessionId = rec.session_id;
  if (typeof sessionId !== "string" || !SESSION_ID_RE.test(sessionId)) {
    return { ok: false, error: "session_id format invalid" };
  }
  const actions = rec.actions;
  if (!Array.isArray(actions)) return { ok: false, error: "actions must be an array" };
  if (actions.length > 3000) return { ok: false, error: "actions too long" };
  for (const step of actions) {
    if (!step || typeof step !== "object" || Array.isArray(step)) {
      return { ok: false, error: "actions must be objects" };
    }
  }
  const first = rec.first;
  if (first !== "a" && first !== "b" && first !== "coin") {
    return { ok: false, error: "first must be a, b, or coin" };
  }
  if (typeof rec.final !== "boolean") return { ok: false, error: "final must be boolean" };
  const winner = rec.winner;
  if (winner !== "a" && winner !== "b" && winner !== null) {
    return { ok: false, error: "winner must be a, b, or null" };
  }
  const decks = parseDecks(rec);
  if (typeof decks === "string") return { ok: false, error: decks };
  const created = rec.created;
  if (typeof created !== "string" || Number.isNaN(Date.parse(created))) {
    return { ok: false, error: "created must be an ISO timestamp" };
  }
  return { ok: true, capture: rec };
}

function capturePath(capture: Record<string, unknown>): string {
  const created = String(capture.created);
  const ym = created.slice(0, 7);
  const gameId = String(capture.game_id);
  const sessionId = String(capture.session_id);
  return `captures/${ym}/${gameId}-${sessionId}.json`;
}

function actionsLen(capture: Record<string, unknown>): number {
  const actions = capture.actions;
  return Array.isArray(actions) ? actions.length : 0;
}

function shouldKeepStored(stored: Record<string, unknown>, incoming: Record<string, unknown>): boolean {
  const storedFinal = Boolean(stored.final);
  const incomingFinal = Boolean(incoming.final);
  if (storedFinal && !incomingFinal) return true;
  const storedLen = actionsLen(stored);
  const incomingLen = actionsLen(incoming);
  if (storedFinal === incomingFinal && storedLen > incomingLen) return true;
  return false;
}

function encodeContent(capture: Record<string, unknown>): string {
  return `${JSON.stringify(capture, null, 1)}\n`;
}

function ghHeaders(token: string): Record<string, string> {
  return {
    Authorization: `Bearer ${token}`,
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "arena-capture",
  };
}

async function ensureBranch(
  cfg: CaptureConfig,
  fetchFn: FetchFn,
): Promise<{ ok: true } | { ok: false; status: number; detail: string }> {
  const refUrl = `https://api.github.com/repos/${cfg.repo}/git/ref/heads/${cfg.branch}`;
  const existing = await fetchFn(refUrl, { headers: ghHeaders(cfg.ghToken) });
  if (existing.ok) return { ok: true };
  if (existing.status !== 404) {
    const detail = await existing.text().catch(() => existing.statusText);
    return { ok: false, status: existing.status, detail };
  }
  const mainRef = await fetchFn(`https://api.github.com/repos/${cfg.repo}/git/ref/heads/main`, {
    headers: ghHeaders(cfg.ghToken),
  });
  if (!mainRef.ok) {
    const detail = await mainRef.text().catch(() => mainRef.statusText);
    return { ok: false, status: mainRef.status, detail };
  }
  const mainJson = (await mainRef.json()) as { object?: { sha?: string } };
  const sha = mainJson.object?.sha;
  if (!sha) return { ok: false, status: 502, detail: "main ref missing sha" };
  const create = await fetchFn(`https://api.github.com/repos/${cfg.repo}/git/refs`, {
    method: "POST",
    headers: { ...ghHeaders(cfg.ghToken), "Content-Type": "application/json" },
    body: JSON.stringify({ ref: `refs/heads/${cfg.branch}`, sha }),
  });
  if (!create.ok && create.status !== 422) {
    const detail = await create.text().catch(() => create.statusText);
    return { ok: false, status: create.status, detail };
  }
  return { ok: true };
}

type GhContent = {
  sha?: string;
  content?: string;
  encoding?: string;
};

async function readStoredCapture(
  cfg: CaptureConfig,
  path: string,
  fetchFn: FetchFn,
): Promise<{ exists: false } | { exists: true; sha: string; capture: Record<string, unknown> }> {
  const url = `https://api.github.com/repos/${cfg.repo}/contents/${path}?ref=${encodeURIComponent(cfg.branch)}`;
  const res = await fetchFn(url, { headers: ghHeaders(cfg.ghToken) });
  if (res.status === 404) return { exists: false };
  if (!res.ok) throw new GhError(res.status, await res.text().catch(() => res.statusText));
  const data = (await res.json()) as GhContent;
  if (!data.sha || !data.content) throw new GhError(502, "missing content from GitHub");
  const raw = Buffer.from(data.content.replace(/\n/g, ""), "base64").toString("utf8");
  const parsed = JSON.parse(raw) as Record<string, unknown>;
  return { exists: true, sha: data.sha, capture: parsed };
}

class GhError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function putCapture(
  cfg: CaptureConfig,
  path: string,
  capture: Record<string, unknown>,
  sha: string | undefined,
  fetchFn: FetchFn,
): Promise<{ commit: string }> {
  const gameId = String(capture.game_id);
  const sessionId = String(capture.session_id);
  const n = actionsLen(capture);
  const tag = capture.final ? "final" : "partial";
  const message = `capture ${gameId}-${sessionId} (${n} actions, ${tag})`;
  const body = {
    message,
    content: Buffer.from(encodeContent(capture), "utf8").toString("base64"),
    branch: cfg.branch,
    ...(sha ? { sha } : {}),
  };
  const url = `https://api.github.com/repos/${cfg.repo}/contents/${path}`;
  const res = await fetchFn(url, {
    method: "PUT",
    headers: { ...ghHeaders(cfg.ghToken), "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new GhError(res.status, await res.text().catch(() => res.statusText));
  const json = (await res.json()) as { commit?: { sha?: string } };
  const commit = json.commit?.sha ?? "";
  return { commit };
}

export async function handleCapturePost(
  request: { method: string; headers: { get(name: string): string | null }; text(): Promise<string> },
  cfg: CaptureConfig,
  fetchFn: FetchFn,
): Promise<Response> {
  if (request.method !== "POST") {
    return jsonResponse(405, { error: "method not allowed" });
  }
  if (!cfg.captureKey || !cfg.ghToken) {
    return jsonResponse(503, { error: "uploads not configured" });
  }
  const key = request.headers.get("x-capture-key");
  if (!constantTimeKeyOk(key, cfg.captureKey)) {
    return jsonResponse(401, { error: "unauthorized" });
  }
  const raw = await request.text();
  const validated = validateCaptureBody(raw);
  if (!validated.ok) {
    return jsonResponse(400, { error: validated.error });
  }
  const capture = validated.capture;
  const path = capturePath(capture);
  if (!path.startsWith("captures/")) {
    return jsonResponse(400, { error: "invalid path" });
  }
  try {
    const branchOk = await ensureBranch(cfg, fetchFn);
    if (!branchOk.ok) {
      return jsonResponse(502, { error: "GitHub branch error", status: branchOk.status });
    }
    let stored = await readStoredCapture(cfg, path, fetchFn);
    if (stored.exists && shouldKeepStored(stored.capture, capture)) {
      return jsonResponse(200, { kept: true });
    }
    const sha = stored.exists ? stored.sha : undefined;
    try {
      const { commit } = await putCapture(cfg, path, capture, sha, fetchFn);
      return jsonResponse(200, { ok: true, path, commit });
    } catch (err) {
      if (err instanceof GhError && (err.status === 409 || err.status === 422)) {
        stored = await readStoredCapture(cfg, path, fetchFn);
        if (stored.exists && shouldKeepStored(stored.capture, capture)) {
          return jsonResponse(200, { kept: true });
        }
        const retrySha = stored.exists ? stored.sha : undefined;
        const { commit } = await putCapture(cfg, path, capture, retrySha, fetchFn);
        return jsonResponse(200, { ok: true, path, commit });
      }
      throw err;
    }
  } catch (err) {
    if (err instanceof GhError) {
      return jsonResponse(502, { error: "GitHub error", status: err.status });
    }
    return jsonResponse(502, { error: "upload failed" });
  }
}

function jsonResponse(status: number, body: Record<string, unknown>): Response {
  const text = JSON.stringify(body);
  return new Response(text, { status, headers: { "content-type": "application/json" } });
}
