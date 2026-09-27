import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const ITSURUGI = "10854110";
const CRYSTALSPAWN = "10631110";

const FANFARE_MODES = [
  "1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.",
  "2. Deal 5 damage to all enemy followers. Recover 1 evolution point.",
];
const EVOLVE_MODES = ["1. Draw 2 cards.", "2. Recover 2 play points."];

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
  await page.locator("#seedInput").fill("42");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckId);
  await page.locator("#redDeckSelect").selectOption(deckId);
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

async function applyEndTurn(page: Page) {
  await page.evaluate(() => {
    const acting = document.getElementById("turnCounter")?.dataset.acting;
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const end = legal.find((a) => "end_turn" in a);
    if (end) window.__arena!.apply(end);
    void acting;
  });
}

async function fastForwardUntilPlayable(page: Page, cardId: string, maxEnds = 24) {
  for (let i = 0; i < maxEnds; i++) {
    const playable = await page.evaluate((id) => {
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      return legal.some((a) => a.play?.card === id);
    }, cardId);
    if (playable) return;
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "end" || phase === "terminal") break;
    await applyEndTurn(page);
    await page.waitForTimeout(50);
  }
}

async function handSize(page: Page, player: "a" | "b" = "a"): Promise<number> {
  return page.evaluate(
    (who) =>
      (window.__arena!.full() as { players: Record<string, { hand: unknown[] }> }).players[who]
        .hand.length,
    player,
  );
}

async function makeHandRoom(page: Page, maxSize: number) {
  for (let guard = 0; guard < 8; guard++) {
    if ((await handSize(page)) <= maxSize) return;
    const played = await page.evaluate((id) => {
      const full = window.__arena!.full() as { players: { a: { field: Array<unknown | null> } } };
      if (full.players.a.field.filter(Boolean).length >= 5) return false;
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const act = legal.find((a) => a.play?.card === id);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    }, CRYSTALSPAWN);
    if (!played) break;
  }
}

async function playCard(page: Page, cardId: string) {
  const ok = await page.evaluate((id) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === id);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, cardId);
  expect(ok, `expected play ${cardId}`).toBeTruthy();
}

async function evolveSlot(page: Page, slot: number) {
  const ok = await page.evaluate((s) => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { slot: number; super: boolean } }>;
    const act = legal.find((a) => a.evolve?.slot === s && !a.evolve?.super);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, slot);
  expect(ok, `expected evolve slot ${slot}`).toBeTruthy();
}

async function modeButtonTexts(page: Page): Promise<string[]> {
  const modal = page.locator(".choice-modal");
  await expect(modal).toBeVisible({ timeout: 5000 });
  return modal.locator(".choice-option").allTextContents();
}

async function clickMode(page: Page, label: string) {
  const btn = page.locator(".choice-modal .choice-option", { hasText: label });
  await expect(btn).toHaveCount(1);
  await page.evaluate((text) => {
    const options = Array.from(
      document.querySelectorAll<HTMLButtonElement>(".choice-modal .choice-option"),
    );
    const hit = options.find((el) => el.textContent?.includes(text));
    hit?.click();
  }, label);
  await expect(page.locator(".choice-modal")).toBeHidden({ timeout: 5000 });
}

test("mode labels: fanfare then evolve after another play", async ({ page }) => {
  await boot(page);
  const deck = await importDeck(page, "itsurugi.json", {
    [ITSURUGI]: 20,
    [CRYSTALSPAWN]: 20,
  });
  await startGame(page, deck);
  await confirmMulligans(page);
  await closeDrawer(page);

  await fastForwardUntilPlayable(page, ITSURUGI);
  await playCard(page, ITSURUGI);
  const fanfareLabels = await modeButtonTexts(page);
  expect(fanfareLabels).toEqual(FANFARE_MODES);
  await clickMode(page, FANFARE_MODES[0]);

  await applyEndTurn(page);
  await applyEndTurn(page);

  await makeHandRoom(page, 7);
  await playCard(page, CRYSTALSPAWN);
  await makeHandRoom(page, 7);
  await evolveSlot(page, 0);
  const evolveLabels = await modeButtonTexts(page);
  expect(evolveLabels).toEqual(EVOLVE_MODES);
  await artShot(page.locator(".choice-modal"), `${ART}/itsurugi_evolve_mode_labels.png`);

  const handBefore = await handSize(page);
  expect(handBefore).toBeLessThanOrEqual(7);
  await clickMode(page, EVOLVE_MODES[0]);
  const handAfter = await handSize(page);
  expect(handAfter).toBe(handBefore + 2);
});
