import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import { fullStrengthPolicyLabel } from "../src/policy-labels.ts";
import { openSettings, turnState } from "./helpers.ts";

const CHEAT_STD = "h0:nodes=6000,info=all";
/** Worker busy-spin so the thinking pill is deterministic on any machine. */
const THINK_QUERY = "localbot=0&botworkerspin=2000";

function readServePolicies(): { strong: string; cheat: string } {
  const servePath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../py/serve.py");
  const text = fs.readFileSync(servePath, "utf8");
  const strongM = text.match(/^DEFAULT_STRONG = "(.*)"$/m);
  const cheatM = text.match(/^DEFAULT_CHEAT = "(.*)"$/m);
  if (!strongM || !cheatM) throw new Error("serve.py policies missing");
  return { strong: strongM[1], cheat: cheatM[1] };
}

function extractServePolicies(source: string): { strong: string; cheat: string } {
  const strongM = source.match(/^DEFAULT_STRONG = "(.*)"$/m);
  const cheatM = source.match(/^DEFAULT_CHEAT = "(.*)"$/m);
  if (!strongM) throw new Error("missing DEFAULT_STRONG in py/serve.py");
  if (!cheatM) throw new Error("missing DEFAULT_CHEAT in py/serve.py");
  const strong = strongM[1];
  const cheat = cheatM[1];
  if (!strong.startsWith("h0:")) {
    throw new Error(`DEFAULT_STRONG must be an h0 spec, got: ${strong}`);
  }
  if (!cheat.startsWith("h0:")) {
    throw new Error(`DEFAULT_CHEAT must be an h0 spec, got: ${cheat}`);
  }
  if (!cheat.split(",").some((part) => part.trim() === "info=all")) {
    throw new Error("DEFAULT_CHEAT must include info=all");
  }
  return { strong, cheat };
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
  const policy = opts.botPolicy ?? "h0";
  await page.locator("#vsBotPolicy").selectOption(policy);
  const human = opts.human ?? "a";
  const other = human === "b" ? "#policyASelect" : "#policyBSelect";
  await page.locator(other).selectOption(policy, { force: true });
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

async function confirmMulligans(page: Page) {
  for (let i = 0; i < 2; i++) {
    const btn = page.locator(".mulligan-confirm-btn:visible");
    if ((await btn.count()) === 0) break;
    await btn.first().click();
    await expect(btn.first()).toBeHidden({ timeout: 5000 }).catch(() => undefined);
  }
}

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

async function playUntilTurnOrEnd(page: Page, minTurn = 6, human: "a" | "b" = "a") {
  for (let guard = 0; guard < 40; guard++) {
    const { phase, turn, acting } = await turnState(page);
    if (phase === "terminal") break;
    if (turn >= minTurn) break;
    if (acting === human) {
      await endHumanTurn(page, human);
    } else {
      await page.waitForTimeout(400);
    }
  }
}

async function actionLog(page: Page): Promise<unknown[]> {
  return page.evaluate(() => window.__arena!.actions());
}

function botSideMoves(log: unknown[], bot: "a" | "b" = "b"): unknown[] {
  return log.filter((step) => {
    const rec = step as Record<string, unknown>;
    if ("mulligan" in rec) return false;
    const inner = Object.values(rec)[0] as { player?: string } | undefined;
    return inner?.player === bot;
  });
}

async function waitForWorkerThinking(page: Page) {
  await expect(page.locator("#botBackendBadge")).toContainText("thinking", { timeout: 30_000 });
  await page.waitForTimeout(650);
  await expect(page.locator("#botThinking")).toBeVisible({ timeout: 5000 });
}

/** Every ancestor with non-visible overflow must fully contain the pill (Playwright visibility ignores clipping). */
async function expectBotThinkingNotClipped(page: Page) {
  const result = await page.evaluate(() => {
    const pill = document.getElementById("botThinking");
    if (!pill || pill.hidden) return { ok: false, reason: "pill hidden" };
    const pillRect = pill.getBoundingClientRect();
    if (pillRect.width <= 0 || pillRect.height <= 0) {
      return { ok: false, reason: "pill has no box" };
    }
    const clips = (v: string) => v !== "visible";
    let el: Element | null = pill.parentElement;
    while (el) {
      const style = getComputedStyle(el);
      const clipX = clips(style.overflowX) || clips(style.overflow);
      const clipY = clips(style.overflowY) || clips(style.overflow);
      if (clipX || clipY) {
        const r = el.getBoundingClientRect();
        if (
          pillRect.top < r.top ||
          pillRect.left < r.left ||
          pillRect.bottom > r.bottom ||
          pillRect.right > r.right
        ) {
          return {
            ok: false,
            reason: "clipped",
            ancestor: `${el.id || el.className || el.tagName}`,
            pillRect: { top: pillRect.top, bottom: pillRect.bottom },
            clipRect: { top: r.top, bottom: r.bottom },
          };
        }
      }
      el = el.parentElement;
    }
    return { ok: true };
  });
  expect(result.ok, JSON.stringify(result)).toBe(true);
}

async function startThinkingScenario(page: Page) {
  await boot(page, "localbot=0&botworkerspin=8000");
  await startVsBot(page, {
    seed: "42",
    first: "b",
    human: "a",
    botPolicy: "h0",
    skipBoot: true,
  });
  await confirmMulligans(page);
  await waitForWorkerThinking(page);
}

test("same moves as main thread (worker vs sync)", async ({ page }) => {
  for (const seed of ["11", "12"]) {
    await startVsBot(page, { seed, botPolicy: "h0", query: "" });
    await confirmHumanMulligan(page, "a");
    await playUntilTurnOrEnd(page, 6, "a");
    const workerLog = await actionLog(page);
    const gen = await page.evaluate(() => window.__arena!.botWorker().generation);
    expect(gen).toBeGreaterThanOrEqual(1);

    await startVsBot(page, { seed, botPolicy: "h0", query: "botworker=0" });
    await confirmHumanMulligan(page, "a");
    await playUntilTurnOrEnd(page, 6, "a");
    const syncLog = await actionLog(page);
    expect(workerLog).toEqual(syncLog);
  }

  await startVsBot(page, { seed: "11", botPolicy: CHEAT_STD });
  await confirmHumanMulligan(page, "a");
  await playUntilTurnOrEnd(page, 6, "a");
  const cheatWorker = await actionLog(page);
  await startVsBot(page, { seed: "11", botPolicy: CHEAT_STD, query: "botworker=0" });
  await confirmHumanMulligan(page, "a");
  await playUntilTurnOrEnd(page, 6, "a");
  const cheatSync = await actionLog(page);
  expect(cheatWorker).toEqual(cheatSync);
});

test.describe("bot thinking pill layout", () => {
  test("desktop 1440×900: pill not clipped by overflow ancestors", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await startThinkingScenario(page);
    await expectBotThinkingNotClipped(page);
  });

  test("iPad landscape 1180×820: pill not clipped by overflow ancestors", async ({ page }) => {
    await page.setViewportSize({ width: 1180, height: 820 });
    await startThinkingScenario(page);
    await expectBotThinkingNotClipped(page);
  });
});

test("page stays live while bot thinks in worker", async ({ page }) => {
  await boot(page, THINK_QUERY);
  await startVsBot(page, {
    seed: "42",
    first: "b",
    human: "a",
    botPolicy: "h0",
    skipBoot: true,
  });
  await confirmMulligans(page);
  await waitForWorkerThinking(page);
  await expect(page.locator("#botThinking")).toContainText(/Bot thinking/);

  const frames = await page.evaluate(async () => {
    let n = 0;
    await new Promise<void>((resolve) => {
      const deadline = performance.now() + 1000;
      const tick = () => {
        n += 1;
        if (performance.now() >= deadline) resolve();
        else requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    });
    return n;
  });
  expect(frames).toBeGreaterThanOrEqual(20);
  await expect(page.locator("#botBackendBadge")).toContainText("thinking");
});

test("undo while worker thinks terminates search", async ({ page }) => {
  await boot(page, THINK_QUERY);
  await startVsBot(page, {
    seed: "43",
    first: "b",
    human: "a",
    botPolicy: "h0",
    skipBoot: true,
  });
  await confirmMulligans(page);
  await waitForWorkerThinking(page);
  const genBefore = await page.evaluate(() => window.__arena!.botWorker().generation);
  await page.keyboard.press("Control+Z");
  await expect
    .poll(async () => {
      const w = await page.evaluate(() => window.__arena!.botWorker());
      return !w.alive && !w.pending;
    })
    .toBeTruthy();
  const before = await actionLog(page);
  await page.waitForTimeout(2000);
  const after = await actionLog(page);
  expect(after).toEqual(before);
  await expect(page.locator("#botThinking")).toBeHidden();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 5000 });
  await page.keyboard.press("Control+Y");
  await page.waitForTimeout(500);
  const genAfterRedo = await page.evaluate(() => window.__arena!.botWorker().generation);
  expect(genAfterRedo).toBeGreaterThan(genBefore);
});

test("new game while worker thinks does not leak old move", async ({ page }) => {
  await boot(page, THINK_QUERY);
  await startVsBot(page, {
    seed: "44",
    first: "b",
    human: "a",
    botPolicy: "h0",
    skipBoot: true,
  });
  await confirmMulligans(page);
  await waitForWorkerThinking(page);
  await openSettings(page);
  await page.locator("#seedInput").fill("44");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmHumanMulligan(page, "a");
  await expect
    .poll(async () => botSideMoves(await actionLog(page)).length, { timeout: 30_000 })
    .toBeGreaterThanOrEqual(1);
  const workerFirst = botSideMoves(await actionLog(page))[0];

  await startVsBot(page, {
    seed: "44",
    first: "b",
    human: "a",
    botPolicy: "h0",
    query: "botworker=0&localbot=0",
  });
  await confirmMulligans(page);
  await expect
    .poll(async () => botSideMoves(await actionLog(page)).length, { timeout: 30_000 })
    .toBeGreaterThanOrEqual(1);
  const syncFirst = botSideMoves(await actionLog(page))[0];
  expect(workerFirst).toEqual(syncFirst);
});

test("worker fallback on forced error matches sync", async ({ page }) => {
  await startVsBot(page, {
    seed: "11",
    first: "b",
    human: "a",
    botPolicy: "h0",
    query: "botworker=fail&localbot=0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).toContainText(/worker error:/, {
    timeout: 30_000,
  });
  const failLog = await actionLog(page);

  await startVsBot(page, {
    seed: "11",
    first: "b",
    human: "a",
    botPolicy: "h0",
    query: "botworker=0&localbot=0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).not.toContainText(/worker error:/, {
    timeout: 30_000,
  });
  await page.waitForTimeout(500);
  const syncLog = await actionLog(page);
  expect(failLog.slice(0, 3)).toEqual(syncLog.slice(0, 3));
});

test("menu lists PC strength from serve.py", async ({ page }) => {
  const { strong, cheat } = readServePolicies();
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  const opts = await page.locator("#vsBotPolicy option").evaluateAll((els) =>
    els.map((el) => ({
      value: (el as HTMLOptionElement).value,
      label: el.textContent ?? "",
    })),
  );
  const cheatIdx = opts.findIndex((o) => o.value === "h0:nodes=6000,info=all");
  expect(cheatIdx).toBeGreaterThanOrEqual(0);
  expect(opts[cheatIdx + 1]).toEqual({ value: strong, label: fullStrengthPolicyLabel(strong, false) });
  expect(opts[cheatIdx + 2]).toEqual({
    value: cheat,
    label: fullStrengthPolicyLabel(cheat, true),
  });
  for (const id of ["#policyASelect", "#policyBSelect"]) {
    const values = await page.locator(`${id} option`).evaluateAll((els) =>
      els.map((el) => (el as HTMLOptionElement).value),
    );
    expect(values).toContain(strong);
    expect(values).toContain(cheat);
  }
});

test("build guard: serve policy extractor", () => {
  expect(() => extractServePolicies('DEFAULT_STRONG = "h0:nodes=1"\n')).toThrow(/DEFAULT_CHEAT/);
  expect(() =>
    extractServePolicies('DEFAULT_STRONG = "h0:nodes=1"\nDEFAULT_CHEAT = "h0:nodes=1"\n'),
  ).toThrow(/info=all/);
  const ok = extractServePolicies(
    'DEFAULT_STRONG = "h0:nodes=1"\nDEFAULT_CHEAT = "h0:nodes=1,info=all"\n',
  );
  expect(ok.strong).toBe("h0:nodes=1");
});
