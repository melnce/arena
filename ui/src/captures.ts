import { version } from "../pkg/arena_wasm.js";
import type { Session } from "./session.ts";
import type { LogStep, SessionConfig } from "./types.ts";
import { isReseedStep } from "./types.ts";

const DB_NAME = "arena-captures";
const STORE = "captures";
const DB_VERSION = 2;
const MAX_RECORDS = 300;
const UPLOAD_INTERVAL_MS = 5 * 60 * 1000;

export type BotBackend = "worker" | "worker-retry" | "main" | "server";

export type BotMetaEntry = {
  ply: number;
  backend: BotBackend;
  ms: number | null;
  value: number | null;
  error?: string;
};

export type GameCapture = {
  v: 1;
  game_id: string;
  seed: string;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  first: string;
  actions: LogStep[];
  policy: string;
  strong: string;
  engine: string;
  humanSide: "a" | "b";
  winner: "a" | "b" | null;
  final: boolean;
  created: string;
  updated: string;
  finished?: string;
  source: "browser";
  session_id: string;
  bot_meta: BotMetaEntry[];
  device: { cores: number; standalone: boolean; ua: string };
};

type StoredRecord = {
  key: string;
  capture: GameCapture;
  uploaded: boolean;
  uploadedFlag: 0 | 1;
  uploadedAt: string | null;
  storedAt: string;
  lastUploadedActionsLen: number | null;
  lastUploadedFinal: boolean | null;
};

type SessionSnapshot = {
  cfg: SessionConfig;
  actions: LogStep[];
  botMeta: BotMetaEntry[];
  sessionId: string;
  created: string;
  phase: string;
  winner: "a" | "b" | null;
};

let idbAvailable = true;
let uploadRejected = false;
let lastError: string | null = null;
let lastUploadAt: string | null = null;
let uploadInFlight = false;
const uploadKeyInFlight = new Set<string>();
let uploadTimer: number | null = null;

let activeSessionId: string | null = null;
let activeCreated: string | null = null;
let botMeta: BotMetaEntry[] = [];

function shortUa(): string {
  const ua = navigator.userAgent;
  return ua.length > 120 ? `${ua.slice(0, 117)}…` : ua;
}

function deviceInfo(): GameCapture["device"] {
  return {
    cores: navigator.hardwareConcurrency ?? 0,
    standalone: window.matchMedia("(display-mode: standalone)").matches,
    ua: shortUa(),
  };
}

export function canonicalDeckJson(deck: Record<string, number>): string {
  const keys = Object.keys(deck).sort();
  const parts = keys.map((k) => `${JSON.stringify(k)}:${JSON.stringify(deck[k])}`);
  return `{${parts.join(",")}}`;
}

export async function sha1Hex(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest("SHA-1", data);
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export async function makeGameId(
  seed: string,
  deckA: Record<string, number>,
  deckB: Record<string, number>,
  first: string,
): Promise<string> {
  const blob = `${canonicalDeckJson(deckA)}|${canonicalDeckJson(deckB)}|${first}`;
  const digest = (await sha1Hex(blob)).slice(0, 8);
  return `${seed}-${digest}`;
}

export function newSessionId(): string {
  const bytes = new Uint8Array(8);
  crypto.getRandomValues(bytes);
  let n = 0n;
  for (const b of bytes) n = (n << 8n) | BigInt(b);
  return n.toString(36).padStart(10, "0").slice(-10);
}

function captureKey(): string | null {
  try {
    return localStorage.getItem("svwb.captureKey");
  } catch {
    return null;
  }
}

export function uploadsEnabled(): boolean {
  try {
    return localStorage.getItem("svwb.captureUploads") !== "0";
  } catch {
    return true;
  }
}

export function setUploadsEnabled(on: boolean): void {
  try {
    localStorage.setItem("svwb.captureUploads", on ? "1" : "0");
  } catch {
    /* private mode */
  }
  refreshCaptureStatusUi();
  if (on) void scheduleUploadDrain();
}

export function saveCaptureKey(key: string): void {
  try {
    localStorage.setItem("svwb.captureKey", key);
  } catch {
    /* */
  }
  uploadRejected = false;
  refreshCaptureStatusUi();
  void scheduleUploadDrain();
}

export function clearCaptureKey(): void {
  try {
    localStorage.removeItem("svwb.captureKey");
  } catch {
    /* */
  }
  refreshCaptureStatusUi();
}

export function applyCaptureKeyFromHash(): void {
  try {
    const hash = location.hash.replace(/^#/, "");
    if (!hash) return;
    const params = new URLSearchParams(hash);
    const key = params.get("capturekey");
    if (!key) return;
    saveCaptureKey(key);
    params.delete("capturekey");
    const rest = params.toString();
    const newHash = rest ? `#${rest}` : "";
    const url = `${location.pathname}${location.search}${newHash}`;
    history.replaceState(null, "", url);
  } catch {
    /* */
  }
}

function stepBody(step: LogStep): Record<string, unknown> {
  if (isReseedStep(step)) return { reseed: step.reseed };
  const raw = JSON.parse(JSON.stringify(step)) as Record<string, unknown>;
  delete raw.value;
  delete raw.bot_value;
  return raw;
}

function stepsMatch(a: LogStep, b: LogStep): boolean {
  return JSON.stringify(stepBody(a)) === JSON.stringify(stepBody(b));
}

function botPolicyForCfg(cfg: SessionConfig): string {
  const human = cfg.humanSide;
  const botSide = human === "a" ? "b" : "a";
  return botSide === "a" ? cfg.policyA : cfg.policyB;
}

function isMulliganAction(action: LogStep): boolean {
  return typeof action === "object" && action !== null && "mulligan" in action;
}

function hasPostMulliganBotAction(s: Session): boolean {
  if (s.game.phase() === "mulligan") return false;
  return s.past.some((p) => !p.human && !isMulliganAction(p.action));
}

function filterBotMeta(actions: LogStep[], meta: BotMetaEntry[]): BotMetaEntry[] {
  return meta.filter((m) => {
    if (m.ply >= actions.length) return false;
    const step = actions[m.ply];
    if (!step) return false;
    const resolved = resolvePlyStep(actions, m.ply);
    if (!resolved) return false;
    return stepsMatch(resolved, step);
  });
}

function resolvePlyStep(actions: LogStep[], ply: number): LogStep | null {
  if (ply >= actions.length) return null;
  const step = actions[ply]!;
  if (isReseedStep(step)) {
    for (let j = ply + 1; j < actions.length; j++) {
      const next = actions[j]!;
      if (!isReseedStep(next)) return next;
    }
    return null;
  }
  return step;
}

function annotateActionsForCapture(actions: LogStep[], meta: BotMetaEntry[]): LogStep[] {
  const copy = JSON.parse(JSON.stringify(actions)) as LogStep[];
  for (const m of meta) {
    if (m.value == null || m.ply >= copy.length) continue;
    const step = copy[m.ply];
    if (!step || typeof step !== "object" || isReseedStep(step)) continue;
    (step as Record<string, unknown>).bot_value = m.value;
  }
  return copy;
}

function captureIsFinal(phase: string): boolean {
  return phase === "terminal";
}

async function buildCaptureFromParts(
  cfg: SessionConfig,
  actions: LogStep[],
  meta: BotMetaEntry[],
  sessionId: string,
  created: string,
  phase: string,
  winner: "a" | "b" | null,
): Promise<GameCapture> {
  const now = new Date().toISOString();
  const final = captureIsFinal(phase);
  const seed = cfg.seed.toString();
  const gameId = await makeGameId(seed, cfg.deckA, cfg.deckB, cfg.first);
  const filtered = filterBotMeta(actions, meta);
  return {
    v: 1,
    game_id: gameId,
    seed,
    deckA: cfg.deckA,
    deckB: cfg.deckB,
    first: cfg.first,
    actions: annotateActionsForCapture(actions, meta),
    policy: botPolicyForCfg(cfg),
    strong: __SERVED_STRONG__,
    engine: version(),
    humanSide: cfg.humanSide,
    winner: final ? winner : null,
    final,
    created,
    updated: now,
    ...(final ? { finished: now } : {}),
    source: "browser",
    session_id: sessionId,
    bot_meta: filtered,
    device: deviceInfo(),
  };
}

async function buildCaptureFromSession(s: Session): Promise<GameCapture> {
  const sessionId = activeSessionId ?? newSessionId();
  const created = activeCreated ?? new Date().toISOString();
  const winner = (s.game.winner() as "a" | "b" | null) ?? null;
  return buildCaptureFromParts(
    s.cfg,
    s.actions,
    botMeta,
    sessionId,
    created,
    s.game.phase(),
    winner,
  );
}

function takeSessionSnapshot(s: Session): SessionSnapshot | null {
  if (s.cfg.mode !== "vs-bot") return null;
  if (!hasPostMulliganBotAction(s)) return null;
  if (!activeSessionId || !activeCreated) return null;
  return {
    cfg: {
      ...s.cfg,
      deckA: { ...s.cfg.deckA },
      deckB: { ...s.cfg.deckB },
    },
    actions: JSON.parse(JSON.stringify(s.actions)) as LogStep[],
    botMeta: botMeta.map((m) => ({ ...m })),
    sessionId: activeSessionId,
    created: activeCreated,
    phase: s.game.phase(),
    winner: (s.game.winner() as "a" | "b" | null) ?? null,
  };
}

async function openDb(): Promise<IDBDatabase | null> {
  if (!idbAvailable) return null;
  try {
    return await new Promise((resolve, reject) => {
      const req = indexedDB.open(DB_NAME, DB_VERSION);
      req.onerror = () => reject(req.error);
      req.onupgradeneeded = () => {
        const db = req.result;
        let store: IDBObjectStore;
        if (!db.objectStoreNames.contains(STORE)) {
          store = db.createObjectStore(STORE, { keyPath: "key" });
        } else {
          store = req.transaction!.objectStore(STORE);
        }
        if (!store.indexNames.contains("byUploaded")) {
          store.createIndex("byUploaded", "uploadedFlag", { unique: false });
        }
      };
      req.onsuccess = () => resolve(req.result);
    });
  } catch {
    idbAvailable = false;
    return null;
  }
}

function normalizeRecord(rec: StoredRecord): StoredRecord {
  if (rec.uploadedFlag === 0 || rec.uploadedFlag === 1) return rec;
  return { ...rec, uploadedFlag: rec.uploaded ? 1 : 0 };
}

async function idbPut(record: StoredRecord): Promise<void> {
  const db = await openDb();
  if (!db) return;
  const row = normalizeRecord(record);
  row.uploadedFlag = row.uploaded ? 1 : 0;
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
      tx.objectStore(STORE).put(row);
    });
  } catch {
    idbAvailable = false;
  } finally {
    db.close();
  }
}

async function idbGet(key: string): Promise<StoredRecord | null> {
  const db = await openDb();
  if (!db) return null;
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).get(key);
      req.onsuccess = () => resolve((req.result as StoredRecord | undefined) ?? null);
      req.onerror = () => reject(req.error);
    });
  } catch {
    idbAvailable = false;
    return null;
  } finally {
    db.close();
  }
}

async function idbGetAll(): Promise<StoredRecord[]> {
  const db = await openDb();
  if (!db) return [];
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).getAll();
      req.onsuccess = () => resolve((req.result as StoredRecord[]) ?? []);
      req.onerror = () => reject(req.error);
    });
  } catch {
    idbAvailable = false;
    return [];
  } finally {
    db.close();
  }
}

async function idbCount(): Promise<number> {
  const db = await openDb();
  if (!db) return 0;
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).count();
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  } catch {
    idbAvailable = false;
    return 0;
  } finally {
    db.close();
  }
}

async function idbCountByUploaded(flag: 0 | 1): Promise<number> {
  const db = await openDb();
  if (!db) return 0;
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const idx = tx.objectStore(STORE).index("byUploaded");
      const req = idx.count(IDBKeyRange.only(flag));
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  } catch {
    return flag === 0 ? 0 : 0;
  } finally {
    db.close();
  }
}

function captureActionsLen(capture: GameCapture): number {
  return capture.actions.length;
}

async function pruneRecords(): Promise<void> {
  const n = await idbCount();
  if (n <= MAX_RECORDS) return;
  const all = await idbGetAll();
  const sorted = [...all].sort((a, b) => {
    if (a.uploaded !== b.uploaded) return a.uploaded ? -1 : 1;
    return a.storedAt.localeCompare(b.storedAt);
  });
  const drop = sorted.slice(0, n - MAX_RECORDS);
  const db = await openDb();
  if (!db) return;
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
      const store = tx.objectStore(STORE);
      for (const rec of drop) store.delete(rec.key);
    });
  } catch {
    idbAvailable = false;
  } finally {
    db.close();
  }
}

async function persistCapture(capture: GameCapture): Promise<void> {
  const key = `${capture.game_id}-${capture.session_id}`;
  const existing = await idbGet(key);
  if (existing?.capture.final && !capture.final) return;

  const record: StoredRecord = {
    key,
    capture,
    uploaded: existing?.uploaded ?? false,
    uploadedFlag: existing?.uploaded ? 1 : 0,
    uploadedAt: existing?.uploadedAt ?? null,
    storedAt: new Date().toISOString(),
    lastUploadedActionsLen: existing?.lastUploadedActionsLen ?? null,
    lastUploadedFinal: existing?.lastUploadedFinal ?? null,
  };

  const len = captureActionsLen(capture);
  if (existing) {
    const grew = len > captureActionsLen(existing.capture);
    const becameFinal = capture.final && !existing.capture.final;
    if (!grew && !becameFinal) {
      record.uploaded = existing.uploaded;
      record.uploadedAt = existing.uploadedAt;
      record.lastUploadedActionsLen = existing.lastUploadedActionsLen;
      record.lastUploadedFinal = existing.lastUploadedFinal;
    } else {
      record.uploaded = false;
      record.uploadedFlag = 0;
    }
  }

  const unchanged =
    existing &&
    captureActionsLen(existing.capture) === len &&
    existing.capture.final === capture.final &&
    existing.capture.winner === capture.winner;
  if (unchanged) return;

  await idbPut(record);
  await pruneRecords();
  void refreshQueueStats().then(() => refreshCaptureStatusUi());
  void scheduleUploadDrain();
}

async function persistSnapshot(snap: SessionSnapshot): Promise<void> {
  const capture = await buildCaptureFromParts(
    snap.cfg,
    snap.actions,
    snap.botMeta,
    snap.sessionId,
    snap.created,
    snap.phase,
    snap.winner,
  );
  await persistCapture(capture);
}

export async function storeCaptureFromSession(s: Session): Promise<void> {
  if (s.cfg.mode !== "vs-bot") return;
  if (!hasPostMulliganBotAction(s) && s.game.phase() !== "terminal") return;
  const capture = await buildCaptureFromSession(s);
  await persistCapture(capture);
}

export function recordBotStep(
  s: Session,
  backend: BotBackend,
  ms: number | null,
  value: number | null,
  error?: string | null,
): void {
  if (s.cfg.mode !== "vs-bot") return;
  const ply = s.actions.length - 1;
  if (ply < 0) return;
  const entry: BotMetaEntry = { ply, backend, ms, value };
  if (error) entry.error = error;
  botMeta.push(entry);
}

export function beginCaptureSession(_cfg: SessionConfig): void {
  activeSessionId = newSessionId();
  activeCreated = new Date().toISOString();
  botMeta = [];
}

export function onSessionStart(cfg: SessionConfig, previous: Session | null): void {
  const snap = previous ? takeSessionSnapshot(previous) : null;
  if (cfg.mode === "vs-bot") beginCaptureSession(cfg);
  else {
    activeSessionId = null;
    activeCreated = null;
    botMeta = [];
  }
  if (snap) void persistSnapshot(snap);
}

export function onGameEnd(s: Session): void {
  void storeCaptureFromSession(s);
}

let cachedStats = { waiting: 0, uploaded: 0, lastError: null as string | null };

export function captureQueueStats(): { waiting: number; uploaded: number; lastError: string | null } {
  return { ...cachedStats, lastError };
}

export async function refreshQueueStats(): Promise<{ waiting: number; uploaded: number; lastError: string | null }> {
  const waiting = await idbCountByUploaded(0);
  const uploaded = await idbCountByUploaded(1);
  cachedStats = { waiting, uploaded, lastError };
  return cachedStats;
}

function shouldSkipUpload(record: StoredRecord): boolean {
  const len = captureActionsLen(record.capture);
  const prevLen = record.lastUploadedActionsLen ?? -1;
  const prevFinal = record.lastUploadedFinal ?? false;
  if (!record.uploaded) return false;
  if (len > prevLen) return false;
  if (record.capture.final && !prevFinal) return false;
  return true;
}

async function uploadOne(record: StoredRecord): Promise<boolean> {
  if (shouldSkipUpload(record)) return true;
  if (uploadKeyInFlight.has(record.key)) return false;
  const key = captureKey();
  if (!key || !uploadsEnabled() || uploadRejected) return false;
  uploadKeyInFlight.add(record.key);
  try {
    const res = await fetch("/api/capture", {
      method: "POST",
      headers: { "content-type": "application/json", "x-capture-key": key },
      body: JSON.stringify(record.capture),
    });
    if (res.status === 401) {
      uploadRejected = true;
      lastError = "upload key rejected";
      refreshCaptureStatusUi();
      return false;
    }
    if (!res.ok) {
      lastError = `upload failed (${res.status})`;
      refreshCaptureStatusUi();
      return false;
    }
    record.uploaded = true;
    record.uploadedFlag = 1;
    record.uploadedAt = new Date().toISOString();
    record.lastUploadedActionsLen = captureActionsLen(record.capture);
    record.lastUploadedFinal = record.capture.final;
    await idbPut(record);
    lastUploadAt = record.uploadedAt;
    lastError = null;
    refreshCaptureStatusUi();
    return true;
  } catch {
    lastError = "offline";
    refreshCaptureStatusUi();
    return false;
  } finally {
    uploadKeyInFlight.delete(record.key);
  }
}

export async function flushCaptures(): Promise<void> {
  if (!captureKey() || !uploadsEnabled() || uploadRejected) return;
  if (uploadInFlight) return;
  uploadInFlight = true;
  try {
    const all = await idbGetAll();
    const pending = all.filter((r) => !r.uploaded).sort((a, b) => a.storedAt.localeCompare(b.storedAt));
    for (const rec of pending) {
      const ok = await uploadOne(rec);
      if (!ok && uploadRejected) break;
      if (!ok && lastError === "offline") break;
    }
    const uploaded = all.filter((r) => r.uploaded);
    for (const rec of uploaded) {
      if (shouldSkipUpload(rec)) continue;
      const ok = await uploadOne(rec);
      if (!ok && uploadRejected) break;
      if (!ok && lastError === "offline") break;
    }
  } finally {
    uploadInFlight = false;
    void refreshQueueStats().then(() => refreshCaptureStatusUi());
  }
}

export function scheduleUploadDrain(): Promise<void> {
  return flushCaptures();
}

function formatStatusLine(stats: { waiting: number; uploaded: number }): string {
  if (!idbAvailable) return "Uploads: unavailable — IndexedDB blocked on this device";
  const key = captureKey();
  if (!key) return "Uploads: off — no key on this device";
  if (!uploadsEnabled()) return `Uploads: off · ${stats.waiting} waiting`;
  if (uploadRejected || lastError === "upload key rejected") {
    return "Uploads: upload key rejected";
  }
  if (lastError) return `Uploads: on · ${stats.waiting} waiting · ${lastError}`;
  const last = lastUploadAt ? new Date(lastUploadAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : "—";
  return `Uploads: on · ${stats.waiting} waiting · last ${last}`;
}

export function refreshCaptureStatusUi(): void {
  void refreshQueueStats().then((stats) => {
    const el = document.getElementById("captureUploadStatus");
    if (el) el.textContent = formatStatusLine(stats);
    const toggle = document.getElementById("captureUploadToggle") as HTMLInputElement | null;
    if (toggle) toggle.checked = uploadsEnabled();
  });
}

export async function exportAllCapturesDownload(): Promise<void> {
  const all = await idbGetAll();
  const captures = all.map((r) => r.capture);
  const blob = new Blob([JSON.stringify(captures, null, 2)], { type: "application/json" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = `arena-captures-${new Date().toISOString().slice(0, 10)}.json`;
  a.click();
  URL.revokeObjectURL(a.href);
}

export function initCaptures(): void {
  applyCaptureKeyFromHash();
  window.addEventListener("online", () => void scheduleUploadDrain());
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden" && sessionRef) {
      void storeCaptureFromSession(sessionRef);
    }
    if (document.visibilityState === "visible") void scheduleUploadDrain();
  });
  window.addEventListener("pagehide", () => {
    if (sessionRef) void storeCaptureFromSession(sessionRef);
  });
  if (uploadTimer != null) window.clearInterval(uploadTimer);
  uploadTimer = window.setInterval(() => {
    if (document.visibilityState === "visible") void scheduleUploadDrain();
  }, UPLOAD_INTERVAL_MS);
  void scheduleUploadDrain();
  refreshCaptureStatusUi();
}

let sessionRef: Session | null = null;

export function setCaptureSessionRef(s: Session | null): void {
  sessionRef = s;
}

/** Test-only: current session bot decision metadata (not persisted until store). */
export function captureBotMetaForTests(): BotMetaEntry[] {
  return botMeta.map((m) => ({ ...m }));
}

/** Test-only: persist the active session capture and wait for IndexedDB. */
export async function storeCaptureNowForTest(): Promise<void> {
  if (!sessionRef) return;
  await storeCaptureFromSession(sessionRef);
}

export function initCapturesSettingsUi(): void {
  const toggle = document.getElementById("captureUploadToggle") as HTMLInputElement | null;
  toggle?.addEventListener("change", (e) => {
    setUploadsEnabled((e.target as HTMLInputElement).checked);
  });
  document.getElementById("captureUploadNowBtn")?.addEventListener("click", () => {
    void flushCaptures();
  });
  document.getElementById("captureExportBtn")?.addEventListener("click", () => {
    void exportAllCapturesDownload();
  });
  document.getElementById("captureUploadKeySave")?.addEventListener("click", () => {
    const input = document.getElementById("captureUploadKeyInput") as HTMLInputElement | null;
    const v = input?.value.trim() ?? "";
    if (v) saveCaptureKey(v);
    if (input) input.value = "";
  });
  document.getElementById("captureUploadKeyClear")?.addEventListener("click", () => {
    clearCaptureKey();
    const input = document.getElementById("captureUploadKeyInput") as HTMLInputElement | null;
    if (input) input.value = "";
  });
  refreshCaptureStatusUi();
}
