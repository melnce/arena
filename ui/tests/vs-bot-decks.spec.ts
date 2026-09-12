import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const FOREST_ONLY = ["10011110", "10011120", "10011130", "10011210", "10012110", "10012120", "10012310"];
const RUNE_ONLY = ["10031110", "10031210", "10031310", "10031320", "10032110", "10032120", "10032310"];

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

async function playerCardIds(page: Page) {
  return page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: {
          deck: Array<{ card: string }>;
          hand: Array<{ card: string }>;
          field: Array<{ card?: string } | null>;
          cemetery: Array<{ card: string }>;
        };
        b: {
          deck: Array<{ card: string }>;
          hand: Array<{ card: string }>;
          field: Array<{ card?: string } | null>;
          cemetery: Array<{ card: string }>;
        };
      };
    };
    const ids = (p: typeof full.players.a) => [
      ...p.deck.map((c) => c.card),
      ...p.hand.map((c) => c.card),
      ...p.cemetery.map((c) => c.card),
      ...p.field.filter((c): c is { card: string } => !!c && !!c.card).map((c) => c.card),
    ];
    return { a: ids(full.players.a), b: ids(full.players.b) };
  });
}

function expectDeck(ids: string[], only: string[], forbidden: string[]) {
  expect(ids.some((id) => only.includes(id))).toBeTruthy();
  expect(ids.some((id) => forbidden.includes(id))).toBeFalsy();
}

async function startVsBot(page: Page, human: "a" | "b" | "coin", seed: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#humanSideSelect").selectOption(human);
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#seedInput").fill(seed);
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

test("vs-bot: blue/red deck mapping stays fixed; labels follow human side", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await expect(page.locator("#blueDeckLabel")).toHaveText("Blue (A):");
  await expect(page.locator("#redDeckLabel")).toHaveText("Red (B):");

  await startVsBot(page, "a", "1");
  await openSettings(page);
  await expect(page.locator("#blueDeckLabel")).toHaveText("Your deck (Blue A)");
  await expect(page.locator("#redDeckLabel")).toHaveText("Bot deck (Red B)");
  await expect(page.locator("#vsBotPolicyLabel")).toHaveText("Bot policy (Red B)");
  await artShot(page.locator(".deck-picker-row"), `${ART}/vs_bot_deck_labels.png`);
  expect(await page.evaluate(() => window.__arena!.humanSide())).toBe("a");
  const first = await playerCardIds(page);
  expectDeck(first.a, FOREST_ONLY, RUNE_ONLY);
  expectDeck(first.b, RUNE_ONLY, FOREST_ONLY);

  await page.locator("#humanSideSelect").selectOption("b");
  await expect(page.locator("#blueDeckLabel")).toHaveText("Bot deck (Blue A)");
  await expect(page.locator("#redDeckLabel")).toHaveText("Your deck (Red B)");
  await expect(page.locator("#vsBotPolicyLabel")).toHaveText("Bot policy (Blue A)");

  await startVsBot(page, "b", "2");
  await openSettings(page);
  await expect(page.locator("#blueDeckLabel")).toHaveText("Bot deck (Blue A)");
  await expect(page.locator("#redDeckLabel")).toHaveText("Your deck (Red B)");
  expect(await page.evaluate(() => window.__arena!.humanSide())).toBe("b");
  const second = await playerCardIds(page);
  expectDeck(second.a, FOREST_ONLY, RUNE_ONLY);
  expectDeck(second.b, RUNE_ONLY, FOREST_ONLY);
});

test("vs-bot coin: session human side matches post-start labels (no re-roll)", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#humanSideSelect").selectOption("coin");
  await expect(page.locator("#blueDeckLabel")).toHaveText("Your deck (coin — decided at start)");
  await expect(page.locator("#redDeckLabel")).toHaveText("Bot deck (Red B)");

  const seen = new Set<string>();
  for (let i = 0; i < 10; i++) {
    await startVsBot(page, "coin", String(i + 1));
    await openSettings(page);
    const side = await page.evaluate(() => window.__arena!.humanSide());
    expect(side === "a" || side === "b").toBeTruthy();
    if (side) seen.add(side);
    const blue = await page.locator("#blueDeckLabel").innerText();
    const red = await page.locator("#redDeckLabel").innerText();
    expect(blue).not.toContain("coin — decided at start");
    if (side === "a") {
      expect(blue).toBe("Your deck (Blue A)");
      expect(red).toBe("Bot deck (Red B)");
      await expect(page.locator("#vsBotPolicyLabel")).toHaveText("Bot policy (Red B)");
    } else {
      expect(blue).toBe("Bot deck (Blue A)");
      expect(red).toBe("Your deck (Red B)");
      await expect(page.locator("#vsBotPolicyLabel")).toHaveText("Bot policy (Blue A)");
    }
    const cards = await playerCardIds(page);
    expectDeck(cards.a, FOREST_ONLY, RUNE_ONLY);
    expectDeck(cards.b, RUNE_ONLY, FOREST_ONLY);
  }
  expect(seen.size).toBeGreaterThan(0);
});
