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
  botPolicy?: string;
  policyA?: string;
  policyB?: string;
}) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode);
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
  if (opts.human) await page.locator("#humanSideSelect").selectOption(opts.human);
  if (opts.botPolicy) {
    await page.locator("#vsBotPolicy").selectOption(opts.botPolicy);
    // Hidden per-side selects stay in sync with the vs-bot dropdown.
    const human = opts.human ?? "a";
    const other = human === "b" ? "#policyASelect" : "#policyBSelect";
    await page.locator(other).selectOption(opts.botPolicy, { force: true });
  }
  if (opts.policyA) await page.locator("#policyASelect").selectOption(opts.policyA, { force: true });
  if (opts.policyB) await page.locator("#policyBSelect").selectOption(opts.policyB, { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function arenaSnap(page: Page) {
  return page.evaluate(() => ({
    hash: window.__arena!.hash(),
    canUndo: window.__arena!.canUndo(),
    canRedo: window.__arena!.canRedo(),
    turn: document.getElementById("turnCounter")?.dataset.turn ?? "",
    phase: document.getElementById("turnCounter")?.dataset.phase ?? "",
    acting: document.getElementById("turnCounter")?.dataset.acting ?? "",
    ply: document.getElementById("turnCounter")?.textContent ?? "",
    readout: document.getElementById("turnReadout")?.textContent ?? "",
    events: document.getElementById("eventLog")?.dataset.count ?? "",
  }));
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

test("hotseat: Ctrl+Z / Ctrl+Y restore Game.hash and button state", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    mode: "hotseat",
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await expect.poll(() => page.evaluate(() => window.__arena?.hash() ?? "")).not.toBe("");

  const snap0 = await arenaSnap(page);
  expect(snap0.hash.length).toBeGreaterThan(0);
  await expect(page.locator("#undoBtn")).toBeDisabled();
  await expect(page.locator("#redoBtn")).toBeDisabled();
  expect(snap0.canUndo).toBe(false);
  expect(snap0.canRedo).toBe(false);

  await confirmMulligans(page);
  await expect(page.locator("#undoBtn")).toBeEnabled({ timeout: 10_000 });
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).not.toBe(snap0.hash);
  const snap2 = await arenaSnap(page);
  expect(snap2.hash).not.toBe(snap0.hash);
  await expect(page.locator("#redoBtn")).toBeDisabled();
  expect(snap2.canUndo).toBe(true);
  expect(snap2.canRedo).toBe(false);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("Control+z");
  await page.keyboard.press("Control+z");
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(snap0.hash);
  const afterUndo = await arenaSnap(page);
  expect(afterUndo).toEqual({ ...snap0, canUndo: false, canRedo: true });
  await expect(page.locator("#undoBtn")).toBeDisabled();
  await expect(page.locator("#redoBtn")).toBeEnabled();

  await page.keyboard.press("Control+y");
  await page.keyboard.press("Control+y");
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(snap2.hash);
  const afterRedo = await arenaSnap(page);
  expect(afterRedo).toEqual({ ...snap2, canUndo: true, canRedo: false });
  await expect(page.locator("#undoBtn")).toBeEnabled();
  await expect(page.locator("#redoBtn")).toBeDisabled();
});

test("vs bot: human A vs h0, bot turn resolves", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  await expect(page.locator("#eventLog")).not.toHaveText("", { timeout: 15_000 });
  const ms = await page.evaluate(() => {
    const t0 = performance.now();
    window.__arena!.botAction("h0", "1");
    return performance.now() - t0;
  });
  console.log(`h0 botAction ${ms.toFixed(1)} ms`);
});

test("watch: h0 vs random reach terminal", async ({ page }) => {
  test.setTimeout(150_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("watch");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#policyASelect").selectOption("h0");
  await page.locator("#policyBSelect").selectOption("random");
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
  await expect(page.locator("#gameOverOverlay")).toBeVisible({ timeout: 120_000 });
});
