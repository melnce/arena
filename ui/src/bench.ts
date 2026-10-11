import type { BenchLine, BenchReply, BenchRequest } from "./bot-worker.ts";
import { decks, parseDeckJson } from "./catalog.ts";
import { publicUrl } from "./base.ts";

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

let benchWorker: Worker | null = null;
let lastBenchJson: unknown = null;
let nextBenchRequestId = 1;

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
    if (benchWorker) {
      benchWorker.terminate();
      benchWorker = null;
    }
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
      benchWorker = new Worker(new URL("./bot-worker.ts", import.meta.url), { type: "module" });
      const w = benchWorker;
      const spec = benchSpec();
      setOut("loading decks…");
      const positions = [];
      for (let i = 0; i < BENCH_POSITIONS.length; i++) {
        const p = BENCH_POSITIONS[i];
        const deckA = await deckJsonFor(p.deckA);
        const deckB = await deckJsonFor(p.deckB);
        positions.push({
          i,
          deckAId: p.deckA,
          deckBId: p.deckB,
          deckA,
          deckB,
          seed: 9100 + i,
          targetTurn: p.targetTurn,
        });
      }
      const id = nextBenchRequestId++;
      const req: BenchRequest = {
        type: "bench",
        id,
        spec,
        drivePolicy: DRIVE_POLICY,
        positions,
      };

      await new Promise<void>((resolve) => {
        const onMsg = (ev: MessageEvent<BenchReply>) => {
          const data = ev.data;
          if (data.type === "bench-progress" && data.id === id) {
            setOut(`position ${data.current} / ${data.total}…`);
          } else if (data.type === "bench-error" && data.id === id) {
            setOut(`error: ${data.message}`);
            w.removeEventListener("message", onMsg);
            stopBench();
            resolve();
          } else if (data.type === "bench-done" && data.id === id) {
            void formatBenchResult(data.lines, spec, data.version).then((text) => {
              setOut(text);
              w.removeEventListener("message", onMsg);
              stopBench();
              resolve();
            });
          }
        };
        w.addEventListener("message", onMsg);
        w.postMessage(req);
      });
      running = false;
    })();
  });
}

async function formatBenchResult(lines: BenchLine[], spec: string, wasmVersion: string): Promise<string> {
  const measured = lines.filter((l) => l.ms != null) as Array<BenchLine & { ms: number }>;
  const msList = measured.map((l) => l.ms);
  const med = msList.length ? median(msList) : 0;
  const max = msList.length ? Math.max(...msList) : 0;
  const total = msList.reduce((a, b) => a + b, 0) / 1000;
  const fp = await fingerprint(lines);
  const cores = navigator.hardwareConcurrency ?? 0;

  const rowLines = lines.map((l) => {
    if (l.turn === "ended") {
      return `${l.i}\t${l.deckA}\t${l.deckB}\tended\tended`;
    }
    return `${l.i}\t${l.deckA}\t${l.deckB}\t${l.turn}\t${l.ms!.toFixed(1)}`;
  });

  lastBenchJson = {
    spec,
    wasmVersion,
    cores,
    medianMs: med,
    maxMs: max,
    totalSec: total,
    fingerprint: fp,
    lines: lines.map((l) => ({
      ...l,
      action: l.actionJson === "ended" ? "ended" : JSON.parse(l.actionJson),
    })),
  };

  return [
    ...rowLines,
    "",
    `median: ${med.toFixed(1)} ms`,
    `max: ${max.toFixed(1)} ms`,
    `total: ${total.toFixed(2)} s`,
    `cores: ${cores}`,
    `wasm: ${wasmVersion}`,
    `spec: ${spec}`,
    `fingerprint: ${fp}`,
  ].join("\n");
}
