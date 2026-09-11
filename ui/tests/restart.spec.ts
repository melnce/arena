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

async function startGame(page: Page) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

test("Restart after 5 actions restores the opening hash and mulligan", async ({ page }) => {
  await boot(page);
  await startGame(page);
  const startHash = await page.evaluate(() => window.__arena!.hash());
  expect(startHash.length).toBeGreaterThan(0);

  for (let i = 0; i < 5; i++) {
    const ok = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
      const order = ["mulligan", "play", "end_turn", "choose", "confirm"];
      const act = order.map((k) => legal.find((a) => k in a)).find(Boolean);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    });
    expect(ok, `action ${i + 1}`).toBeTruthy();
  }
  expect(await page.evaluate(() => window.__arena!.hash())).not.toBe(startHash);
  expect(await page.evaluate(() => window.__arena!.canUndo())).toBe(true);

  await page.locator("#restartRailBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan", {
    timeout: 10_000,
  });
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(startHash);
  expect(await page.evaluate(() => window.__arena!.canUndo())).toBe(false);

  await openSettings(page);
  await expect(page.locator("#restartGameBtn")).toBeEnabled();
  await page.locator("#restartGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan");
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(startHash);
});
