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

async function startGame(page: Page, deckId: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckId);
  await page.locator("#redDeckSelect").selectOption(deckId);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

test("no rail/drawer Restart; Rematch (same seed) restores opening hash and mulligan", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await boot(page);
  const tiny = await importDeck(page, "tiny.json", { "10001110": 8 });
  await startGame(page, tiny);
  await expect(page.locator("#restartRailBtn")).toHaveCount(0);
  await expect(page.locator("#restartGameBtn")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Restart" })).toHaveCount(0);

  const startHash = await page.evaluate(() => window.__arena!.hash());
  expect(startHash.length).toBeGreaterThan(0);

  for (let i = 0; i < 40; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    const ended = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
      const act = legal.find((a) => "end_turn" in a);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    });
    if (!ended) break;
  }
  await expect(page.locator("#gameOverOverlay")).toBeVisible({ timeout: 10_000 });
  await expect(page.locator("#rematchSameSeedBtn")).toHaveText("Rematch (same seed)");
  await expect(page.locator("#rematchNewSeedBtn")).toHaveText("Rematch (new seed)");
  await expect(page.locator("#newGameFromOver")).toHaveText("New Game");

  await page.locator("#rematchSameSeedBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan", {
    timeout: 10_000,
  });
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(startHash);
  await expect(page.locator("#gameOverOverlay")).toBeHidden();
  await expect(page.locator("#restartRailBtn")).toHaveCount(0);
});
