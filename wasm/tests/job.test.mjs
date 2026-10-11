#!/usr/bin/env node
// H0 job path vs botAction / botActionValue on wasm (separate Game per piece).
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const pkgDir = path.resolve(
  process.env.ARENA_WASM_PKG || path.join(here, "../pkg-node"),
);

const require = createRequire(import.meta.url);
const { Game } = require(path.join(pkgDir, "arena_wasm.js"));

const STRONG =
  "h0:nodes=4000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";
const PLAY = "h0:nodes=300";

function loadDeck(stem) {
  const file = stem.endsWith(".json") ? stem : `${stem}.json`;
  return fs.readFileSync(path.join(repo, "oracle/decks", file), "utf8");
}

function poolsMeta() {
  const pools = JSON.parse(
    fs.readFileSync(path.join(repo, "oracle/decks/POOLS.json"), "utf8"),
  );
  return pools.meta.filter((s) => s !== "meta-rune-test-subject");
}

function replayToTurn(game, targetTurn, seed) {
  let g = game;
  let turn = g.turn();
  let n = 0;
  while (turn < targetTurn && g.winner() === null && n < 500) {
    const legal = JSON.parse(g.legal());
    if (!legal.length) break;
    const pol = g.active() === "a" ? PLAY : PLAY;
    const act = JSON.parse(g.botAction(pol, seed + n));
    g = g.clone();
    g.apply(JSON.stringify(act));
    turn = g.turn();
    n += 1;
  }
  return g;
}

function collectPositions() {
  const decks = poolsMeta().slice(0, 6);
  const out = [];
  for (let i = 0; i < decks.length; i++) {
    const deck = loadDeck(decks[i]);
    for (const targetTurn of [5, 7]) {
      const seed = 20261011 + i * 97 + targetTurn * 13;
      const g0 = new Game(seed, deck, deck, "a");
      const g = replayToTurn(g0, targetTurn, seed);
      if (g.winner() !== null) continue;
      const legal = JSON.parse(g.legal());
      if (!legal.length) continue;
      out.push({ label: `${decks[i]}@t${targetTurn}`, game: g, seed: seed + 999 });
      if (out.length >= 10) return out;
    }
  }
  return out;
}

function parseStep(json) {
  return JSON.parse(json);
}

function runJobSeparateGames(baseGame, policy, seed) {
  const coordinator = baseGame.clone();
  let step = parseStep(coordinator.h0JobStart(policy, seed));
  while (step.step !== "done") {
    if (step.step === "kill") {
      const detGame = baseGame.clone();
      const det = JSON.parse(detGame.h0KillDet(policy, seed));
      step = parseStep(coordinator.h0JobKillDet(JSON.stringify(det)));
      if (step.step === "await_roll") {
        const rollGame = baseGame.clone();
        const roll = JSON.parse(rollGame.h0KillRoll(policy, seed));
        step = parseStep(coordinator.h0JobKillRoll(JSON.stringify(roll)));
      }
    } else if (step.step === "worlds") {
      const worlds = [];
      for (let r = 0; r < step.budgets.length; r++) {
        const wg = baseGame.clone();
        worlds.push(
          JSON.parse(wg.h0JobWorld(policy, seed, r, step.budgets[r])),
        );
      }
      step = parseStep(coordinator.h0JobFinish(JSON.stringify(worlds)));
    } else if (step.step === "await_roll") {
      const rollGame = baseGame.clone();
      const roll = JSON.parse(rollGame.h0KillRoll(policy, seed));
      step = parseStep(coordinator.h0JobKillRoll(JSON.stringify(roll)));
    } else {
      throw new Error(`unexpected step ${JSON.stringify(step)}`);
    }
  }
  return step;
}

const cases = collectPositions();
assert.ok(cases.length >= 10, `need 10 positions, got ${cases.length}`);

for (const { label, game, seed } of cases) {
  const direct = JSON.parse(game.botAction(STRONG, seed));
  const valued = JSON.parse(game.botActionValue(STRONG, seed));
  assert.deepEqual(direct, valued.action, `${label} botAction vs botActionValue`);
  const job = runJobSeparateGames(game, STRONG, seed);
  assert.deepEqual(direct, job.action, `${label} botAction vs job path`);
  if (valued.value !== null) {
    assert.equal(job.value, valued.value, `${label} value`);
  }
}

console.log(`h0 job wasm: ${cases.length} positions ok`);
