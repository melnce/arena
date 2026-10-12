import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import { openSettings, turnState } from "./helpers.ts";

/** CI-sized world-split spec (same family as wasm job tests). */
export const WORLD_SPLIT_SPEC =
  "h0:nodes=4000,k=8,horizon=3,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";

const THINK_CANCEL_QUERY = `localbot=0&botcores=4&botworkerspin=30000`;

async function ensurePolicyOption(page: Page, selector: string, value: string) {
  await page.evaluate(
    ({ sel, val }) => {
      const el = document.querySelector(sel) as HTMLSelectElement | null;
      if (!el) return;
      if (![...el.options].some((o) => o.value === val)) {
        const opt = document.createElement("option");
        opt.value = val;
        opt.textContent = val.length > 48 ? `${val.slice(0, 45)}…` : val;
        el.appendChild(opt);
      }
      el.value = val;
    },
    { sel: selector, val: value },
  );
}

async function boot(page: Page, query = "") {
  await page.goto(`/${query ? `?${query}` : ""}`);
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function startVsBot(
  page: Page,
  opts: {
    seed: string;
    human?: "a" | "b";
    first?: "a" | "b";
    botPolicy?: string;
    query?: string;
    skipBoot?: boolean;
  },
) {
  if (!opts.skipBoot) {
    if (opts.query) await boot(page, opts.query);
    else await boot(page);
  }
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill(opts.seed);
  await page.locator("#firstSelect").selectOption(opts.first ?? "a");
  await page.locator("#humanSideSelect").selectOption(opts.human ?? "a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  const policy = opts.botPolicy ?? WORLD_SPLIT_SPEC;
  await ensurePolicyOption(page, "#vsBotPolicy", policy);
  const human = opts.human ?? "a";
  const other = human === "b" ? "#policyASelect" : "#policyBSelect";
  await ensurePolicyOption(page, other, policy);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmHumanMulligan(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "#blueMulliganConfirm" : "#redMulliganConfirm";
  const btn = page.locator(id);
  await expect(btn).toBeVisible({ timeout: 15_000 });
  await btn.click();
  await expect(btn).toBeHidden({ timeout: 5000 });
}

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

async function playUntilTurnOrEnd(page: Page, minTurn = 6, human: "a" | "b" = "a") {
  for (let guard = 0; guard < 300; guard++) {
    const { phase, turn, acting } = await turnState(page);
    if (phase === "terminal") break;
    if (turn >= minTurn && acting === human) break;
    if (acting === human) {
      await endHumanTurn(page, human);
    } else {
      await page.waitForTimeout(500);
    }
  }
}

async function actionLog(page: Page): Promise<unknown[]> {
  return page.evaluate(() => window.__arena!.actions());
}

async function waitForWorkerThinking(page: Page) {
  await expect(page.locator("#botBackendBadge")).toContainText("thinking", { timeout: 30_000 });
  await page.waitForTimeout(650);
  await expect(page.locator("#botThinking")).toBeVisible({ timeout: 5000 });
}

function loadKillFixture(): Record<string, unknown> {
  const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
  const raw = fs.readFileSync(
    path.join(repo, "engine/tests/fixtures/lethal/review15-g4-fuse-kill.json"),
    "utf8",
  );
  const log = JSON.parse(raw) as Record<string, unknown>;
  log.v = 1;
  log.kind = "replay-log";
  log.seed = String(log.seed);
  log.deckAId = log.deckAId ?? "imported-a";
  log.deckBId = log.deckBId ?? "imported-b";
  const ply = (log.ply as number | undefined) ?? (log.actions as unknown[]).length;
  log.actions = (log.actions as unknown[]).slice(0, ply);
  return log;
}

test("same moves for any core count", async ({ page }) => {
  const baselineBySeed: Record<string, unknown[]> = {};
  for (const seed of ["11", "12"]) {
    await boot(page, "localbot=0&botworker=0");
    await startVsBot(page, {
      seed,
      botPolicy: WORLD_SPLIT_SPEC,
      skipBoot: true,
    });
    await confirmHumanMulligan(page, "a");
    await playUntilTurnOrEnd(page, 6, "a");
    baselineBySeed[seed] = await actionLog(page);
  }

  for (const cores of ["1", "2", "3", "4"]) {
    for (const seed of ["11", "12"]) {
      await boot(page, `localbot=0&botcores=${cores}`);
      await startVsBot(page, {
        seed,
        botPolicy: WORLD_SPLIT_SPEC,
        skipBoot: true,
      });
      await confirmHumanMulligan(page, "a");
      await playUntilTurnOrEnd(page, 6, "a");
      const log = await actionLog(page);
      expect(log).toEqual(baselineBySeed[seed]);
    }
  }
});

test("pool splits work across cores", async ({ page }) => {
  const hw = await page.evaluate(() => navigator.hardwareConcurrency ?? 1);

  await startVsBot(page, {
    seed: "11",
    first: "b",
    human: "a",
    botPolicy: WORLD_SPLIT_SPEC,
    query: "localbot=0&botcores=4",
  });
  await confirmHumanMulligan(page, "a");
  const sawBusy = await page.evaluate(async () => {
    for (let i = 0; i < 18_000; i++) {
      if (window.__arena!.botPool().busy >= 2) return true;
      await new Promise((r) => setTimeout(r, 10));
    }
    return false;
  });
  expect(sawBusy).toBe(true);

  await boot(
    page,
    `localbot=0&benchspec=${encodeURIComponent(WORLD_SPLIT_SPEC)}&botcores=${Math.min(8, hw)}`,
  );
  await openSettings(page);
  await page.locator("#benchRunBtn").click();
  await expect(page.locator("#benchOutput")).toContainText("speed-up:", { timeout: 300_000 });
  const text = (await page.locator("#benchOutput").textContent()) ?? "";
  expect(text).toMatch(/same moves/i);
  if (hw >= 2) {
    const speedUp = Number(text.match(/speed-up:\s*([\d.]+)/)?.[1] ?? "0");
    expect(speedUp, `speed-up ${speedUp} on ${hw} cores`).toBeGreaterThan(1.3);
  }
});

test("cancel terminates every pool worker", async ({ page }) => {
  await boot(page, THINK_CANCEL_QUERY);
  await startVsBot(page, {
    seed: "43",
    first: "b",
    human: "a",
    botPolicy: WORLD_SPLIT_SPEC,
    skipBoot: true,
  });
  await confirmHumanMulligan(page, "a");
  await waitForWorkerThinking(page);
  await page.keyboard.press("Control+Z");
  await expect
    .poll(
      async () => {
        const p = await page.evaluate(() => window.__arena!.botPool());
        return p.alive === 0;
      },
      { timeout: 500 },
    )
    .toBe(true);
  const before = await actionLog(page);
  await page.waitForTimeout(2000);
  const after = await actionLog(page);
  expect(after).toEqual(before);
});

test("kill step matches single worker", async ({ page }) => {
  const policy = WORLD_SPLIT_SPEC;
  const fixture = loadKillFixture();
  const actions = fixture.actions as unknown[];
  let replayable = { ...fixture, actions: [] as unknown[] };
  await boot(page, "localbot=0");
  for (let ply = 1; ply <= actions.length; ply++) {
    const candidate = { ...fixture, actions: actions.slice(0, ply) };
    const ok = await page.evaluate((log) => window.__arena!.tryReplayLog(log), candidate);
    if (!ok.ok) break;
    replayable = candidate;
  }
  // Capture logs may diverge from wasm opening draws; use the longest UI-replayable prefix.
  expect(replayable.actions.length).toBeGreaterThan(2);
  const acting = await page.evaluate((log) => {
    window.__arena!.loadLog(log);
    return document.getElementById("turnCounter")?.getAttribute("data-acting");
  }, replayable);
  const humanSide = acting === "a" ? "b" : "a";

  async function botMoveWith(query: string) {
    await boot(page, query);
    await page.evaluate((log) => window.__arena!.loadLog(log), replayable);
    await page.evaluate(
      ({ pol, human }) => window.__arena!.armVsBot({ humanSide: human, botPolicy: pol }),
      { pol: policy, human: humanSide },
    );
    await expect
      .poll(async () => (await actionLog(page)).length, { timeout: 180_000 })
      .toBeGreaterThan(replayable.actions.length);
    const log = await actionLog(page);
    return log[replayable.actions.length];
  }

  const poolBotMove = await botMoveWith("localbot=0&botcores=4");
  const syncBotMove = await botMoveWith("localbot=0&botworker=0");
  expect(poolBotMove).toEqual(syncBotMove);
});
