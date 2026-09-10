#!/usr/bin/env node
// Replay arena-trace JSONL through the WASM Game and assert snapshot/hash match.
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const pkgDir = path.resolve(
  process.env.ARENA_WASM_PKG || path.join(here, "../pkg-node"),
);
const deckAPath = path.resolve(
  repo,
  process.env.ARENA_DECK_A || "oracle/decks/basic-forest.json",
);
const deckBPath = path.resolve(
  repo,
  process.env.ARENA_DECK_B || "oracle/decks/basic-rune.json",
);
const seedCount = Number(process.env.ARENA_WASM_SEEDS || 20);
const firstSeed = Number(process.env.ARENA_WASM_SEED0 || 1);
const first = process.env.ARENA_FIRST || "coin";
const traceDir = path.resolve(
  process.env.ARENA_TRACE_DIR || path.join(repo, "target/wasm-traces"),
);
const traceBin =
  process.env.ARENA_TRACE_BIN || path.join(repo, "target/release/arena-trace");

const require = createRequire(import.meta.url);
const { Game, bundleInfo, version } = require(
  path.join(pkgDir, "arena_wasm.js"),
);

function fnv1a64(bytes) {
  let h = 0xcbf29ce484222325n;
  const mul = 0x0100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (const b of bytes) {
    h ^= BigInt(b);
    h = (h * mul) & mask;
  }
  return h.toString(10);
}

function ensureTraces() {
  const last = path.join(traceDir, `${firstSeed + seedCount - 1}.jsonl`);
  if (fs.existsSync(last)) return;
  fs.mkdirSync(traceDir, { recursive: true });
  const bin = fs.existsSync(traceBin) ? traceBin : "arena-trace";
  const r = spawnSync(
    bin,
    [
      "--seed",
      String(firstSeed),
      "--games",
      String(seedCount),
      "--deck-a",
      deckAPath,
      "--deck-b",
      deckBPath,
      "--out",
      traceDir,
      "--first",
      first,
    ],
    { cwd: repo, encoding: "utf8" },
  );
  if (r.status !== 0) {
    throw new Error(
      `arena-trace failed (${r.status}): ${r.stderr || r.stdout}`,
    );
  }
}

function deepEqual(a, b, pathSoFar) {
  const d = diff(a, b, pathSoFar || "");
  if (d) {
    throw new Error(`snapshot mismatch at ${d.path}: wasm=${d.got} native=${d.want}`);
  }
}

function diff(a, b, p) {
  if (a === b) return null;
  if (a === null || b === null || typeof a !== typeof b) {
    return { path: p, got: JSON.stringify(a), want: JSON.stringify(b) };
  }
  if (typeof a !== "object") {
    return { path: p, got: JSON.stringify(a), want: JSON.stringify(b) };
  }
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) {
      return { path: p, got: JSON.stringify(a), want: JSON.stringify(b) };
    }
    for (let i = 0; i < a.length; i++) {
      const d = diff(a[i], b[i], `${p}[${i}]`);
      if (d) return d;
    }
    return null;
  }
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const k of keys) {
    const d = diff(a[k], b[k], p ? `${p}.${k}` : k);
    if (d) return d;
  }
  return null;
}

const deckA = fs.readFileSync(deckAPath, "utf8");
const deckB = fs.readFileSync(deckBPath, "utf8");
ensureTraces();

const info = JSON.parse(bundleInfo());
assert.ok(info.cards > 0, "bundle has cards");
console.log(
  `version=${version()} cards=${info.cards} crests=${info.crests} bytes=${info.bytes}`,
);

let games = 0;
let actions = 0;
for (let s = firstSeed; s < firstSeed + seedCount; s++) {
  const file = path.join(traceDir, `${s}.jsonl`);
  const lines = fs.readFileSync(file, "utf8").trim().split("\n");
  const header = JSON.parse(lines[0]);
  assert.equal(header.seed, s);
  const game = new Game(s, deckA, deckB, first);
  try {
    for (let i = 1; i < lines.length; i++) {
      const line = JSON.parse(lines[i]);
      game.apply(JSON.stringify(line.action));
      const snap = JSON.parse(game.snapshot());
      deepEqual(snap, line.state, `seed ${s} i=${line.i}`);
      const snapBytes = new TextEncoder().encode(game.snapshot());
      assert.equal(
        game.hash(),
        fnv1a64(snapBytes),
        `hash() vs FNV of snapshot() at seed ${s} i=${line.i}`,
      );
      actions += 1;
    }
    games += 1;
  } finally {
    game.free();
  }
}

console.log(
  `ok: ${games} seeds, ${actions} actions, wasm snapshot+hash == arena-trace`,
);
assert.equal(games, seedCount);
