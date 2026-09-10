import { expect, test, type Page } from "@playwright/test";

async function openSettings(page: Page) {
  const drawer = page.locator("#settingsDrawer");
  if (!(await drawer.evaluate((el) => el.classList.contains("open")))) {
    await page.locator("#settingsToggle").click();
  }
  await expect(drawer).toHaveClass(/open/);
}

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function startGame(page: Page, opts: {
  mode: string;
  seed?: string;
  first?: string;
  deckA?: string;
  deckB?: string;
  human?: string;
}) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode);
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
  if (opts.human) await page.locator("#humanSideSelect").selectOption(opts.human);
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

test("hotseat: seed 1, both mulligans, play, end turn", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    mode: "hotseat",
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await confirmMulligans(page);
  const before = await page.locator("#turnCounter").innerText();
  const playable = page.locator(".hand-zone .card.legal-play");
  if (await playable.count()) {
    await playable.first().click();
  }
  const end = page.locator("#endTurnBlue:visible, #endTurnRed:visible");
  await expect(end).toHaveCount(1, { timeout: 5000 });
  await end.first().click();
  await expect(page.locator("#turnCounter")).not.toHaveText(before, { timeout: 5000 });
  const log = await page.locator("#eventLog").innerText();
  expect(log.length).toBeGreaterThan(0);
});

test("vs bot: human A vs random, bot turn resolves", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 5000,
  });
  await expect(page.locator("#eventLog")).not.toHaveText("", { timeout: 5000 });
});

test("watch: two random bots reach terminal", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("watch");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#watchSpeed").evaluate((el) => {
    (el as HTMLInputElement).value = "20";
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /.+/, {
    timeout: 15_000,
  });
  const play = page.locator("#watchPlayBtn");
  if (await play.isVisible()) await play.click();
  await expect(page.locator("#gameOverOverlay")).toBeVisible({ timeout: 60_000 });
});
