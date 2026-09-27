import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

type PositionLog = {
  v: 1;
  kind: "replay-log";
  seed: string;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  deckAId: string;
  deckBId: string;
  first: string;
  actions: unknown[];
  at?: number;
  name?: string;
  turn?: number;
  savedAt?: string;
};

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function startHotseat(page: Page) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("7");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmMulligans(page: Page) {
  for (let i = 0; i < 2; i++) {
    const btn = page.locator(".mulligan-confirm-btn").locator("visible=true");
    if (await btn.count()) {
      await btn.first().click();
      await expect(btn).toBeHidden({ timeout: 5000 }).catch(() => undefined);
    }
  }
}

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function endTurnApply(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => "end_turn" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
}

async function playSomeTurns(page: Page, turns: number) {
  for (let i = 0; i < turns; i++) {
    const ended = await endTurnApply(page);
    if (!ended) break;
  }
}

/** Apply one legal play or attack when available (distinct from end_turn). */
async function applyPlayOrAttack(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act =
      legal.find((a) => "play" in a) ??
      legal.find((a) => "attack" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
}

/** Drive main phase: prefer plays/attacks, then end turn. */
async function driveMain(page: Page, maxSteps = 40) {
  for (let i = 0; i < maxSteps; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    if (phase === "choice") {
      await page.evaluate(() => {
        const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
        const act = legal.find((a) => "choose" in a);
        if (act) window.__arena!.apply(act);
      });
      continue;
    }
    if (await applyPlayOrAttack(page)) continue;
    if (await endTurnApply(page)) continue;
    break;
  }
}

function actionKey(step: unknown): string {
  return JSON.stringify(step);
}

async function loadLog(page: Page, log: PositionLog) {
  await page.evaluate((raw) => window.__arena!.loadLog(raw), log);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main|choice|terminal/, {
    timeout: 15_000,
  });
}

async function fullState(page: Page) {
  return page.evaluate(() => window.__arena!.full());
}

async function prefixState(page: Page, log: PositionLog, at: number) {
  const prefix: PositionLog = { ...log, actions: log.actions.slice(0, at) };
  delete prefix.at;
  await loadLog(page, prefix);
  return fullState(page);
}

test("replay at: import opens at step with redo tail and export round-trips", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startHotseat(page);
  await confirmMulligans(page);
  await closeDrawer(page);
  await playSomeTurns(page, 8);

  const fullLog = (await page.evaluate(() => window.__arena!.exportLog())) as PositionLog;
  expect(fullLog.actions.length).toBeGreaterThan(4);
  expect(fullLog.at).toBeUndefined();

  const fullHash = await page.evaluate(() => window.__arena!.hash());
  const at = Math.max(2, Math.floor(fullLog.actions.length / 2));

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const undos = fullLog.actions.length - at;
  for (let i = 0; i < undos; i++) {
    await page.keyboard.press("Control+z");
  }
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(at);
  await expect(page.locator("#redoBtn")).toBeEnabled();
  await expect(page.locator("#replayStepReadout")).toHaveText(`step ${at} / ${fullLog.actions.length}`);

  const exported = (await page.evaluate(() => window.__arena!.exportPositionLog())) as PositionLog;
  expect(exported.at).toBe(at);
  expect(exported.actions).toEqual(fullLog.actions);

  const prefix = await prefixState(page, fullLog, at);
  const logAt: PositionLog = { ...fullLog, at };
  await loadLog(page, logAt);
  const atState = await fullState(page);
  expect(atState).toEqual(prefix);
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(at);
  await expect(page.locator("#redoBtn")).toBeEnabled();
  await expect(page.locator("#replayStepReadout")).toHaveText(`step ${at} / ${fullLog.actions.length}`);

  const beforeRedo = await page.evaluate(() => window.__arena!.hash());
  await page.keyboard.press("Control+y");
  expect(await page.evaluate(() => window.__arena!.hash())).not.toBe(beforeRedo);

  for (let i = at + 1; i < fullLog.actions.length; i++) {
    await page.keyboard.press("Control+y");
  }
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
  await expect(page.locator("#redoBtn")).toBeDisabled();
  await expect(page.locator("#replayStepReadout")).toBeHidden();

  const roundTrip = (await page.evaluate(() => window.__arena!.exportPositionLog())) as PositionLog;
  expect(roundTrip.at).toBeUndefined();
  expect(roundTrip.actions).toEqual(fullLog.actions);
});

test("replay at: at=0, at=length, no-at legacy, invalid at toast", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startHotseat(page);
  await confirmMulligans(page);
  await closeDrawer(page);
  await playSomeTurns(page, 6);

  const fullLog = (await page.evaluate(() => window.__arena!.exportLog())) as PositionLog;
  const fullHash = await page.evaluate(() => window.__arena!.hash());
  const len = fullLog.actions.length;

  await loadLog(page, { ...fullLog, at: 0 });
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(0);
  await expect(page.locator("#redoBtn")).toBeEnabled();
  const startState = await fullState(page);
  const emptyPrefix = await prefixState(page, fullLog, 0);
  expect(startState).toEqual(emptyPrefix);

  await loadLog(page, { ...fullLog, at: len });
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
  await expect(page.locator("#redoBtn")).toBeDisabled();

  await loadLog(page, { ...fullLog });
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
  await expect(page.locator("#redoBtn")).toBeDisabled();

  await loadLog(page, { ...fullLog, at: -1 });
  await expect(page.locator("#actionToast")).toHaveClass(/visible/);
  await expect(page.locator("#actionToast")).toContainText(/Invalid at/);
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);

  await loadLog(page, { ...fullLog, at: 1.5 });
  await expect(page.locator("#actionToast")).toContainText(/Invalid at/);
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);

  await loadLog(page, { ...fullLog, at: len + 5 });
  await expect(page.locator("#actionToast")).toContainText(/Invalid at/);
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
});

test("replay at: reseed in redo tail opens at end with toast", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startHotseat(page);
  await confirmMulligans(page);
  await closeDrawer(page);
  for (let i = 0; i < 20; i++) {
    const turn = await page.evaluate(
      () => (window.__arena!.full() as { turn: number }).turn,
    );
    if (turn >= 3) break;
    await endTurnApply(page);
  }
  await page.keyboard.press("F6");
  await endTurnApply(page);
  const branchHash = await page.evaluate(() => window.__arena!.hash());
  await page.keyboard.press("F8");
  await endTurnApply(page);

  const fullLog = (await page.evaluate(() => window.__arena!.exportLog())) as PositionLog;
  const fullHash = await page.evaluate(() => window.__arena!.hash());
  const reseedIdx = fullLog.actions.findIndex(
    (s) => typeof s === "object" && s !== null && "reseed" in s,
  );
  expect(reseedIdx).toBeGreaterThan(0);
  expect(reseedIdx).toBeLessThan(fullLog.actions.length);

  await loadLog(page, { ...fullLog, at: reseedIdx });
  await expect(page.locator("#actionToast")).toHaveClass(/visible/);
  await expect(page.locator("#actionToast")).toContainText(/Reseed in redo tail/);
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
  await expect(page.locator("#redoBtn")).toBeDisabled();
});

test("replay at: distinct redo tail exports in chronological order", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  await startHotseat(page);
  await confirmMulligans(page);
  await closeDrawer(page);
  await driveMain(page, 50);

  const fullLog = (await page.evaluate(() => window.__arena!.exportLog())) as PositionLog;
  const fullHash = await page.evaluate(() => window.__arena!.hash());
  expect(fullLog.actions.length).toBeGreaterThan(6);

  let at = -1;
  for (let n = 3; n <= 5; n++) {
    const start = fullLog.actions.length - n;
    const keys = fullLog.actions.slice(start).map(actionKey);
    if (new Set(keys).size >= 3) {
      at = start;
      break;
    }
  }
  expect(at, "need a redo tail of 3+ distinct actions").toBeGreaterThan(0);
  const tailLen = fullLog.actions.length - at;

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  for (let i = 0; i < tailLen; i++) await page.keyboard.press("Control+z");
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(at);
  await expect(page.locator("#redoBtn")).toBeEnabled();

  const logAt: PositionLog = { ...fullLog, at };
  await loadLog(page, logAt);

  const exported = (await page.evaluate(() => window.__arena!.exportPositionLog())) as PositionLog;
  expect(exported.at).toBe(at);
  expect(exported.actions).toEqual(fullLog.actions);

  await loadLog(page, exported);
  for (let i = at; i < fullLog.actions.length; i++) {
    await page.keyboard.press("Control+y");
  }
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  for (let i = 0; i < tailLen; i++) await page.keyboard.press("Control+z");
  await openSettings(page);
  page.once("dialog", (d) => d.accept("distinct-tail"));
  await page.locator("#savePositionBtn").click();
  const saved = (await page.evaluate(() => window.__arena!.savedPosition())) as PositionLog;
  expect(saved.at).toBe(at);
  expect(saved.actions).toEqual(fullLog.actions);

  await page.locator("#loadPositionBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /main|choice/, {
    timeout: 15_000,
  });
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(at);
  for (let i = at; i < fullLog.actions.length; i++) {
    await page.keyboard.press("Control+y");
  }
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(fullHash);
});

test("default exportLog matches bot payload shape when redo tail exists", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill("3");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#vsBotPolicy").selectOption("h0");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await expect(page.locator("#blueMulliganConfirm")).toBeVisible({ timeout: 15_000 });
  await page.locator("#blueMulliganConfirm").click();
  await expect(page.locator("#blueMulliganConfirm")).toBeHidden({ timeout: 10_000 });
  await closeDrawer(page);

  const ended = await endTurnApply(page);
  expect(ended).toBeTruthy();
  await endTurnApply(page);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("Control+z");
  const applied = await page.evaluate(() => window.__arena!.actions().length);
  expect(applied).toBeGreaterThan(0);
  await expect(page.locator("#redoBtn")).toBeEnabled();

  const exportLog = (await page.evaluate(() => window.__arena!.exportLog())) as PositionLog;
  expect(exportLog.at).toBeUndefined();
  expect(exportLog.actions.length).toBe(applied);

  const exportPos = (await page.evaluate(() => window.__arena!.exportPositionLog())) as PositionLog;
  expect(exportPos.at).toBe(applied);
  expect(exportPos.actions.length).toBeGreaterThan(applied);
});
