import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

const ONE_COST = "10052110";

async function boot(page: Page) {
  await page.goto("/?localbot=0");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function importDeck(page: Page, name: string, cards: Record<string, number>) {
  await openSettings(page);
  await page.locator("#deckImportFileInput").setInputFiles({
    name,
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(cards)),
  });
  const id = `import-${name.replace(/\.json$/i, "")}`;
  await expect(page.locator("#blueDeckSelect")).toHaveValue(id, { timeout: 10_000 });
  return id;
}

async function startVsBot(
  page: Page,
  opts: {
    seed: string;
    humanDeck: string;
    botDeck: string;
    human?: "a" | "b";
    first?: "a" | "b" | "coin";
    botPolicy?: string;
    hideBotHand?: boolean;
  },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill(opts.seed);
  await page.locator("#firstSelect").selectOption(opts.first ?? "a");
  await page.locator("#humanSideSelect").selectOption(opts.human ?? "a");
  await page.locator("#blueDeckSelect").selectOption(opts.humanDeck);
  await page.locator("#redDeckSelect").selectOption(opts.botDeck);
  const policy = opts.botPolicy ?? "first-legal";
  await page.locator("#vsBotPolicy").selectOption(policy);
  const human = opts.human ?? "a";
  const other = human === "b" ? "#policyASelect" : "#policyBSelect";
  await page.locator(other).selectOption(policy, { force: true });
  const hide = opts.hideBotHand ?? true;
  if (hide) await page.locator("#hideBotHandToggle").check();
  else await page.locator("#hideBotHandToggle").uncheck();
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

async function waitHumanTurn(page: Page, human: "a" | "b" = "a", timeout = 30_000) {
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", human, { timeout });
}

async function waitBotPlay(page: Page, timeout = 30_000) {
  await expect
    .poll(
      async () => {
        const lines = (await page.locator("#eventLog").innerText()).split("\n");
        for (let i = lines.length - 1; i >= 0; i--) {
          const line = lines[i]?.trim();
          if (!line) continue;
          try {
            const ev = JSON.parse(line) as { play?: { player?: string } };
            if (ev.play?.player === "b") return true;
          } catch {
            /* skip */
          }
        }
        return false;
      },
      { timeout },
    )
    .toBeTruthy();
}

async function eventLogText(page: Page): Promise<string> {
  return page.locator("#eventLog").innerText();
}

async function setupGame(page: Page) {
  const humanDeck = await importDeck(page, "resume-human.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "resume-bot.json", { [ONE_COST]: 40 });
  await startVsBot(page, {
    seed: "8484",
    humanDeck,
    botDeck,
    human: "a",
    first: "a",
    botPolicy: "first-legal",
    hideBotHand: true,
  });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
}

async function referenceLog(page: Page): Promise<string> {
  await setupGame(page);
  await endHumanTurn(page, "a");
  await waitHumanTurn(page, "a");
  return await eventLogText(page);
}

test.describe("vs-bot bot resumes after redo", () => {
  test("reference: full bot turn reaches deterministic event log", async ({ page }) => {
    test.setTimeout(90_000);
    await boot(page);
    const R = await referenceLog(page);
    expect(R.length).toBeGreaterThan(0);
    expect(R).toContain('"player":"b"');
  });

  test("slow redo resumes the bot", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    const R = await referenceLog(page);

    await setupGame(page);
    await endHumanTurn(page, "a");
    await waitBotPlay(page);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
    await page.waitForTimeout(600);
    await page.keyboard.press("Control+y");
    await waitHumanTurn(page, "a", 10_000);
    expect(await eventLogText(page)).toBe(R);
  });

  test("fast redo resumes the bot without duplicate actions", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    const R = await referenceLog(page);

    await setupGame(page);
    await endHumanTurn(page, "a");
    await waitBotPlay(page);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await page.keyboard.press("Control+y");
    await waitHumanTurn(page, "a", 10_000);
    expect(await eventLogText(page)).toBe(R);
  });

  test("redo stack is kept while human is acting", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    await setupGame(page);
    await endHumanTurn(page, "a");
    await waitBotPlay(page);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
    await expect(page.locator("#redoBtn")).toBeEnabled();
    const logAfterUndo = await eventLogText(page);
    await page.waitForTimeout(1500);
    expect(await eventLogText(page)).toBe(logAfterUndo);
    await expect(page.locator("#redoBtn")).toBeEnabled();
  });

  test("redo stack kept when the bot is to act", async ({ page }) => {
    test.setTimeout(90_000);
    await boot(page);
    const humanDeck = await importDeck(page, "resume-human-4b.json", { [ONE_COST]: 40 });
    const botDeck = await importDeck(page, "resume-bot-4b.json", { [ONE_COST]: 40 });
    await startVsBot(page, {
      seed: "8484",
      humanDeck,
      botDeck,
      human: "a",
      first: "b",
      botPolicy: "first-legal",
      hideBotHand: true,
    });
    await closeDrawer(page);
    await expect
      .poll(async () => (await page.evaluate(() => window.__arena!.actions().length)) >= 1, {
        timeout: 15_000,
      })
      .toBeTruthy();
    await page.waitForTimeout(600);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan");
    expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(0);
    await expect(page.locator("#redoBtn")).toBeEnabled();
    await page.waitForTimeout(1500);
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b");
    expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(0);
    await expect(page.locator("#redoBtn")).toBeEnabled();
    await page.keyboard.press("Control+y");
    expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(1);
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
  });

  test("watch mode stays paused after undo/redo", async ({ page }) => {
    test.setTimeout(90_000);
    await boot(page);
    const deck = await importDeck(page, "resume-watch.json", { [ONE_COST]: 40 });
    await openSettings(page);
    await page.locator("#modeSelect").selectOption("watch");
    await page.locator("#seedInput").fill("1111");
    await page.locator("#blueDeckSelect").selectOption(deck);
    await page.locator("#redDeckSelect").selectOption(deck);
    await page.locator("#policyASelect").selectOption("first-legal", { force: true });
    await page.locator("#policyBSelect").selectOption("first-legal", { force: true });
    await page.locator("#startGameBtn").click();
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
      timeout: 15_000,
    });
    await page.locator("#watchSpeed").evaluate((el) => {
      (el as HTMLInputElement).value = "5";
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await page.locator("#watchPlayBtn").click();
    await page.waitForTimeout(1000);
    const logBefore = await eventLogText(page);
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await page.keyboard.press("Control+y");
    const logAfterRedo = await eventLogText(page);
    await page.waitForTimeout(1500);
    expect(logAfterRedo).toBe(logBefore);
    expect(await eventLogText(page)).toBe(logBefore);
  });
});
