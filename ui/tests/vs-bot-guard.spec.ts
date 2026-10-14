import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import { openSettings, turnState } from "./helpers.ts";

const SPIN = "localbot=0&botworkerspin=12000";

function readServeStrong(): string {
  const servePath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../py/serve.py");
  const text = fs.readFileSync(servePath, "utf8");
  const m = text.match(/^DEFAULT_STRONG = "(.*)"$/m);
  if (!m) throw new Error("DEFAULT_STRONG missing");
  return m[1];
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
    await boot(page, opts.query ?? "");
  }
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill(opts.seed);
  await page.locator("#firstSelect").selectOption(opts.first ?? "a");
  await page.locator("#humanSideSelect").selectOption(opts.human ?? "a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  const policy = opts.botPolicy ?? readServeStrong();
  await page.locator("#vsBotPolicy").selectOption(policy);
  const human = opts.human ?? "a";
  const other = human === "b" ? "#policyASelect" : "#policyBSelect";
  await page.locator(other).selectOption(policy, { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmHumanMulligan(page: Page, human: "a" | "b") {
  const id = human === "a" ? "#blueMulliganConfirm" : "#redMulliganConfirm";
  const btn = page.locator(id);
  await expect(btn).toBeVisible({ timeout: 15_000 });
  await btn.click();
  await expect(btn).toBeHidden({ timeout: 5000 });
}

async function confirmMulligans(page: Page) {
  for (let i = 0; i < 2; i++) {
    const btn = page.locator(".mulligan-confirm-btn:visible");
    if ((await btn.count()) === 0) break;
    await btn.first().click();
    await expect(btn.first()).toBeHidden({ timeout: 5000 }).catch(() => undefined);
  }
}

async function actionLog(page: Page): Promise<unknown[]> {
  return page.evaluate(() => window.__arena!.actions());
}

async function waitBotThinking(page: Page) {
  await expect(page.locator("#botBackendBadge")).toContainText("thinking", { timeout: 30_000 });
}

async function scenarioEndTurnGuard(
  page: Page,
  opts: { human: "a" | "b"; first: "a" | "b"; seed: string },
) {
  await boot(page, SPIN);
  await startVsBot(page, { ...opts, skipBoot: true });
  await confirmMulligans(page);
  const bot = opts.human === "a" ? "b" : "a";
  const humanBtn = opts.human === "a" ? "#endTurnBlue" : "#endTurnRed";
  if (opts.first === opts.human) {
    await page.locator(`${humanBtn}:visible`).click();
    await expect.poll(async () => (await turnState(page)).acting).toBe(bot);
  }
  await waitBotThinking(page);
  const botBtn = opts.human === "a" ? "#endTurnRed" : "#endTurnBlue";
  await expect(page.locator(botBtn)).toBeHidden();
  await expect(page.locator(humanBtn)).toBeVisible();
  await expect(page.locator(humanBtn)).toBeDisabled();
  const before = await actionLog(page);
  await page.locator(humanBtn).click({ force: true });
  expect(await actionLog(page)).toEqual(before);
  await expect
    .poll(async () => (await actionLog(page)).length, { timeout: 45_000 })
    .toBeGreaterThan(before.length);
}

test("vs-bot: human cannot end bot turn while bot thinks (human a, bot first)", async ({ page }) => {
  await scenarioEndTurnGuard(page, { human: "a", first: "b", seed: "9001" });
});

test("vs-bot: human cannot end bot turn while bot thinks (human a, human first)", async ({
  page,
}) => {
  await scenarioEndTurnGuard(page, { human: "a", first: "a", seed: "9002" });
});

test("vs-bot: bonus PP and bot choices do not commit for bot", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page, `${SPIN}&botcores=1`);
  await startVsBot(page, { seed: "9010", first: "b", human: "a", skipBoot: true });
  await confirmMulligans(page);
  await waitBotThinking(page);
  const before = await actionLog(page);
  await page.locator("#bonusPpBtn").click({ force: true });
  expect(await actionLog(page)).toEqual(before);
  await expect
    .poll(async () => (await actionLog(page)).length, { timeout: 60_000 })
    .toBeGreaterThan(before.length);
});

test("hotseat: end turn and bonus PP still work", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("9020");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#startGameBtn").click();
  await confirmMulligans(page);
  const before = await actionLog(page);
  await page.locator("#endTurnBlue:visible").click();
  await expect
    .poll(async () => (await actionLog(page)).length)
    .toBeGreaterThan(before.length);
  const toggled = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ bonus_pp?: { player: string } }>;
    const act = legal.find((a) => a.bonus_pp);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(toggled).toBe(true);
});

test("worker failonce retries and records error", async ({ page }) => {
  await boot(page, "localbot=0&botworker=failonce&botcores=4");
  await startVsBot(page, { seed: "9030", first: "b", human: "a", botPolicy: "h0", skipBoot: true });
  await confirmMulligans(page);
  await expect
    .poll(
      async () => {
        const meta = await page.evaluate(() => window.__arena!.captureBotMeta());
        return meta.find((m) => m.backend === "worker-retry");
      },
      { timeout: 60_000 },
    )
    .toBeTruthy();
  const retried = (await page.evaluate(() => window.__arena!.captureBotMeta())).find(
    (m) => m.backend === "worker-retry",
  );
  expect(retried!.error).toMatch(/forced worker failure/);
  await expect.poll(async () => (await turnState(page)).acting).toBe("a", { timeout: 60_000 });
  await page.locator("#endTurnBlue:visible").click();
  await expect
    .poll(async () => {
      const m = await page.evaluate(() => window.__arena!.captureBotMeta());
      return m.some((e) => e.backend === "worker" && !e.error);
    }, { timeout: 90_000 })
    .toBeTruthy();
});

test("main-thread fallback shows busy overlay and blocks end turn", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page, `localbot=0&botworker=fail&botcores=1&botslowmain=1500`);
  await startVsBot(page, {
    seed: "9040",
    first: "b",
    human: "a",
    botPolicy: readServeStrong(),
    skipBoot: true,
  });
  await confirmMulligans(page);
  const overlaySeen = expect
    .poll(
      async () =>
        page.locator("#botBusyOverlay").evaluate((el) => getComputedStyle(el).display !== "none"),
      { timeout: 120_000, intervals: [10, 20, 50] },
    )
    .toBe(true);
  await overlaySeen;
  const before = await actionLog(page);
  const box = await page.locator("#endTurnBlue").boundingBox();
  if (box) {
    await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  }
  expect(await actionLog(page)).toEqual(before);
  await expect(page.locator("#botBusyOverlay")).toBeHidden({ timeout: 120_000 });
});

test("undo during a slow main-thread decision", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page, "localbot=0&botworker=0&botslowmain=1500");
  await startVsBot(page, {
    seed: "9060",
    first: "a",
    human: "a",
    botPolicy: "h0",
    skipBoot: true,
  });
  await confirmMulligans(page);
  await expect.poll(async () => (await turnState(page)).acting).toBe("a");
  const logBeforeEnd = await actionLog(page);
  await page.locator("#endTurnBlue:visible").click();
  await expect
    .poll(
      async () =>
        page.locator("#botBusyOverlay").evaluate((el) => getComputedStyle(el).display !== "none"),
      { timeout: 60_000 },
    )
    .toBe(true);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("Control+z");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
  await expect
    .poll(
      async () =>
        page.locator("#botBusyOverlay").evaluate((el) => getComputedStyle(el).display === "none"),
      { timeout: 15_000 },
    )
    .toBe(true);
  expect(await actionLog(page)).toEqual(logBeforeEnd);
  await expect(page.locator("#redoBtn")).toBeEnabled();
  await page.keyboard.press("Control+y");
  await expect
    .poll(
      async () =>
        page.locator("#botBusyOverlay").evaluate((el) => getComputedStyle(el).display !== "none"),
      { timeout: 60_000 },
    )
    .toBe(true);
  await expect.poll(async () => (await turnState(page)).acting).toBe("a", { timeout: 120_000 });
  const log = (await actionLog(page)) as Array<Record<string, unknown>>;
  const endIdx = log.findIndex((step) => "end_turn" in step);
  expect(endIdx).toBeGreaterThanOrEqual(0);
  for (let i = endIdx + 1; i < log.length; i++) {
    const step = log[i];
    const inner = Object.values(step)[0] as { player?: string } | undefined;
    if (inner?.player) expect(inner.player).toBe("b");
  }
});

test("bot bonus PP is recorded in bot_meta", async ({ page }) => {
  test.setTimeout(240_000);
  await boot(page, "localbot=0&botworker=0");
  for (const seed of ["9050", "9051", "9052", "42", "43"]) {
    await startVsBot(page, {
      seed,
      first: "a",
      human: "a",
      botPolicy: "h0",
      skipBoot: true,
    });
    await confirmMulligans(page);
    const humanBtn = "#endTurnBlue";
    for (let t = 0; t < 24; t++) {
      const { acting, phase } = await turnState(page);
      if (phase === "terminal") break;
      if (acting === "a") {
        const btn = page.locator(`${humanBtn}:visible`);
        if (await btn.count()) await btn.click();
      } else {
        await page.waitForTimeout(300);
      }
      const log = (await actionLog(page)) as Array<Record<string, unknown>>;
      const bonusPly = log.findIndex((step) => {
        if (!("bonus_pp" in step)) return false;
        const inner = step.bonus_pp as { player?: string };
        return inner?.player === "b";
      });
      if (bonusPly >= 0) {
        const meta = await page.evaluate(() => window.__arena!.captureBotMeta());
        expect(meta.some((m) => m.ply === bonusPly)).toBe(true);
        return;
      }
    }
  }
  throw new Error("no bot bonus_pp in tried seeds");
});
