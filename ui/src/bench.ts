import { Game, version } from "../pkg/arena_wasm.js";
import type { BenchLine } from "./bot-worker.ts";
import {
  autoBotCores,
  benchDecideAtPosition,
  configuredBotCores,
} from "./bot-client.ts";
import { decks, parseDeckJson } from "./catalog.ts";
import { publicUrl } from "./base.ts";
import type { LogStep } from "./types.ts";

const BENCH_POSITIONS: Array<{
  deckA: string;
  deckB: string;
  targetTurn: number;
}> = [
  { deckA: "meta-abyss-midrange", deckB: "meta-rune-test-subject", targetTurn: 6 },
  { deckA: "meta-rune-test-subject", deckB: "meta-sword-rally", targetTurn: 4 },
  { deckA: "meta-haven-evo", deckB: "meta-abyss-midrange", targetTurn: 8 },
  { deckA: "meta-dragon-ramp", deckB: "meta-portal-cutthroat", targetTurn: 4 },
  { deckA: "meta-forest-combo", deckB: "meta-haven-amulet", targetTurn: 9 },
  { deckA: "meta-sword-loot", deckB: "meta-rune-spell", targetTurn: 8 },
  { deckA: "meta-portal-evo", deckB: "meta-dragon-aggro", targetTurn: 5 },
  { deckA: "meta-abyss-aggro", deckB: "meta-rune-crystal", targetTurn: 7 },
  { deckA: "meta-portal-af", deckB: "meta-forest-combo", targetTurn: 5 },
  { deckA: "meta-rune-spell", deckB: "meta-abyss-midrange", targetTurn: 9 },
];

const DRIVE_POLICY = "h0:nodes=300";

let lastBenchJson: unknown = null;
let benchAbort: AbortController | null = null;

export function benchResult(): unknown | null {
  return lastBenchJson;
}

function benchSpec(): string {
  try {
    const override = new URLSearchParams(location.search).get("benchspec");
    if (override) return override;
  } catch {
    /* ignore */
  }
  return __SERVED_STRONG__;
}

async function deckJsonFor(id: string): Promise<string> {
  const entry = decks().find((d) => d.id === id);
  if (!entry) throw new Error(`unknown deck ${id}`);
  const raw = await fetch(publicUrl(`decks/${entry.file}`)).then((r) => r.text());
  return JSON.stringify(parseDeckJson(raw));
}

function median(nums: number[]): number {
  const sorted = [...nums].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) return (sorted[mid - 1] + sorted[mid]) / 2;
  return sorted[mid];
}

async function fingerprint(lines: BenchLine[]): Promise<string> {
  const payload = lines.map((l) => l.actionJson).join("\n");
  const data = new TextEncoder().encode(payload);
  const hash = await crypto.subtle.digest("SHA-256", data);
  const hex = Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return hex.slice(0, 12);
}

type BenchPrepared = {
  i: number;
  deckAId: string;
  deckBId: string;
  body: {
    seed: string;
    deckA: string;
    deckB: string;
    first: string;
    actions: LogStep[];
    hash: string;
  };
  botSeed: string;
  turn: number | "ended";
};

function prepareBenchPosition(
  i: number,
  seed: number,
  deckA: string,
  deckB: string,
  deckAId: string,
  deckBId: string,
  targetTurn: number,
): BenchPrepared | null {
  const game = new Game(String(seed), deckA, deckB, "a");
  const actions: LogStep[] = [];
  let n = 0;
  try {
    for (let step = 0; step < 400; step++) {
      if (game.phase() === "terminal") return null;
      const turn = game.turn();
      if (turn >= targetTurn && game.phase() === "main") {
        const legal = JSON.parse(game.legal()) as unknown[];
        if (legal.length >= 3) {
          return {
            i,
            deckAId,
            deckBId,
            body: {
              seed: String(seed),
              deckA,
              deckB,
              first: "a",
              actions,
              hash: game.hash(),
            },
            botSeed: String(seed + n),
            turn: game.turn(),
          };
        }
      }
      const raw = game.botAction(DRIVE_POLICY, String(seed + n));
      actions.push(JSON.parse(raw) as LogStep);
      game.apply(raw);
      n += 1;
    }
    return null;
  } finally {
    game.free();
  }
}

async function runBenchLines(
  prepared: Array<BenchPrepared | null>,
  spec: string,
  cores: number,
  onProgress: (current: number, total: number) => void,
  signal: AbortSignal,
): Promise<BenchLine[]> {
  const lines: BenchLine[] = [];
  const total = prepared.length;
  for (let idx = 0; idx < total; idx++) {
    if (signal.aborted) throw new Error("aborted");
    onProgress(idx + 1, total);
    const pos = prepared[idx];
    if (!pos) {
      lines.push({
        i: idx,
        deckA: BENCH_POSITIONS[idx].deckA,
        deckB: BENCH_POSITIONS[idx].deckB,
        turn: "ended",
        ms: null,
        actionJson: "ended",
      });
      continue;
    }
    const decided = await benchDecideAtPosition(pos.body, spec, pos.botSeed, cores);
    lines.push({
      i: pos.i,
      deckA: pos.deckAId,
      deckB: pos.deckBId,
      turn: pos.turn,
      ms: decided.ms,
      actionJson: decided.actionJson,
    });
  }
  return lines;
}

function statsFromLines(lines: BenchLine[]): { med: number; max: number; total: number } {
  const measured = lines.filter((l) => l.ms != null) as Array<BenchLine & { ms: number }>;
  const msList = measured.map((l) => l.ms);
  const med = msList.length ? median(msList) : 0;
  const max = msList.length ? Math.max(...msList) : 0;
  const total = msList.reduce((a, b) => a + b, 0) / 1000;
  return { med, max, total };
}

export function initBenchUi(): void {
  const runBtn = document.getElementById("benchRunBtn");
  const stopBtn = document.getElementById("benchStopBtn");
  const copyBtn = document.getElementById("benchCopyBtn");
  const out = document.getElementById("benchOutput");
  if (!runBtn || !stopBtn || !copyBtn || !out) return;

  let running = false;

  const setOut = (text: string) => {
    out.textContent = text;
  };

  const stopBench = () => {
    benchAbort?.abort();
    benchAbort = null;
    running = false;
  };

  stopBtn.addEventListener("click", () => {
    stopBench();
    setOut((out.textContent ?? "") + "\n(stopped)");
  });

  copyBtn.addEventListener("click", async () => {
    if (!lastBenchJson) return;
    const text = JSON.stringify(lastBenchJson, null, 2);
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      /* ignore */
    }
  });

  runBtn.addEventListener("click", () => {
    if (running) return;
    void (async () => {
      running = true;
      stopBench();
      benchAbort = new AbortController();
      const signal = benchAbort.signal;
      const spec = benchSpec();
      const poolCores = configuredBotCores();
      setOut("loading decks…");
      const prepared: Array<BenchPrepared | null> = [];
      for (let i = 0; i < BENCH_POSITIONS.length; i++) {
        const p = BENCH_POSITIONS[i];
        const deckA = await deckJsonFor(p.deckA);
        const deckB = await deckJsonFor(p.deckB);
        prepared.push(
          prepareBenchPosition(i, 9100 + i, deckA, deckB, p.deckA, p.deckB, p.targetTurn),
        );
      }

      try {
        setOut("1 core: position 0 / 10…");
        const lines1 = await runBenchLines(
          prepared,
          spec,
          1,
          (c, t) => setOut(`1 core: position ${c} / ${t}…`),
          signal,
        );
        setOut(`pool (${poolCores} cores): position 0 / 10…`);
        const linesPool = await runBenchLines(
          prepared,
          spec,
          poolCores,
          (c, t) => setOut(`pool (${poolCores} cores): position ${c} / ${t}…`),
          signal,
        );
        const text = await formatBenchResult(lines1, linesPool, spec, version(), poolCores);
        setOut(text);
      } catch (err) {
        if (String(err).includes("aborted")) {
          setOut((out.textContent ?? "") + "\n(stopped)");
        } else {
          setOut(`error: ${err instanceof Error ? err.message : String(err)}`);
        }
      }
      running = false;
      benchAbort = null;
    })();
  });
}

async function formatBenchResult(
  lines1: BenchLine[],
  linesPool: BenchLine[],
  spec: string,
  wasmVersion: string,
  poolCores: number,
): Promise<string> {
  const s1 = statsFromLines(lines1);
  const sPool = statsFromLines(linesPool);
  const fp1 = await fingerprint(lines1);
  const fpPool = await fingerprint(linesPool);
  const sameMoves = fp1 === fpPool;
  const speedUp = s1.med > 0 ? sPool.med > 0 ? s1.med / sPool.med : 0 : 0;
  const cores = navigator.hardwareConcurrency ?? 0;

  const rowLines = linesPool.map((l) => {
    if (l.turn === "ended") {
      return `${l.i}\t${l.deckA}\t${l.deckB}\tended\tended`;
    }
    return `${l.i}\t${l.deckA}\t${l.deckB}\t${l.turn}\t${l.ms!.toFixed(1)}`;
  });

  lastBenchJson = {
    spec,
    wasmVersion,
    cores,
    poolCores,
    oneCore: { medianMs: s1.med, maxMs: s1.max, totalSec: s1.total, fingerprint: fp1 },
    pool: { medianMs: sPool.med, maxMs: sPool.max, totalSec: sPool.total, fingerprint: fpPool },
    speedUp,
    sameMoves,
    fingerprint: fp1,
    lines: linesPool.map((l) => ({
      ...l,
      action: l.actionJson === "ended" ? "ended" : JSON.parse(l.actionJson),
    })),
  };

  const fpLabel = sameMoves
    ? `fingerprint: ${fp1} (same moves)`
    : `fingerprint: ${fp1} — pool ${fpPool} DIFFERENT`;

  return [
    ...rowLines,
    "",
    `1 core — median: ${s1.med.toFixed(1)} ms, max: ${s1.max.toFixed(1)} ms, total: ${s1.total.toFixed(2)} s`,
    `pool (${poolCores}) — median: ${sPool.med.toFixed(1)} ms, max: ${sPool.max.toFixed(1)} ms, total: ${sPool.total.toFixed(2)} s`,
    `speed-up: ${speedUp.toFixed(2)}×`,
    `cores: ${cores} (auto ${autoBotCores()}, pool ${poolCores})`,
    `wasm: ${wasmVersion}`,
    `spec: ${spec}`,
    fpLabel,
  ].join("\n");
}
