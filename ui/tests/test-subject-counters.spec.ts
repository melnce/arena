import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

/** Seed 136: Sephie + two Test Subjects in hand; A reaches k ≥ 5 by turn 5. */
const SEED = "136";

const TEST_SUBJECT = "10931110";
const SEPHIE = "10934110";
const SCHOLAR = "10933110";
const ENAMORED = "10932110";
const HUMANE_LOVE = "10932310";
const OBSIDIAN_RAVEN = "10933310";

const FOLLOWER_SUMMONERS = [SEPHIE, SCHOLAR, ENAMORED];
const NO_LINE_CARDS = [TEST_SUBJECT, HUMANE_LOVE, OBSIDIAN_RAVEN];

const META_DECK = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "../../oracle/decks/meta-rune-test-subject.json"), "utf8"),
) as Record<string, number>;

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
  await page.locator("#seedInput").fill(SEED);
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

async function endTurn(page: Page) {
  const before = await page.locator("#turnCounter").getAttribute("data-acting");
  await page.evaluate(() => {
    document.querySelector(".choice-modal")?.remove();
    const acting = document.getElementById("turnCounter")?.dataset.acting;
    const id = acting === "a" ? "endTurnBlue" : "endTurnRed";
    (document.getElementById(id) as HTMLButtonElement | null)?.click();
  });
  await expect(page.locator("#turnCounter")).not.toHaveAttribute("data-acting", before ?? "", {
    timeout: 8000,
  });
}

async function resolveChoices(page: Page) {
  for (let i = 0; i < 8; i++) {
    const chose = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ choose?: number }>;
      const act = legal.find((a) => a.choose != null);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    });
    if (!chose) break;
  }
}

async function playCard(page: Page, card: string) {
  const ok = await page.evaluate((id) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === id);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, card);
  expect(ok, `expected play ${card}`).toBeTruthy();
  await resolveChoices(page);
}

async function enterCount(page: Page, side: "a" | "b" = "a"): Promise<number> {
  return page.evaluate(
    ([player, id]) => {
      const full = window.__arena!.full() as {
        players: { a: { enter_counts: Record<string, number> }; b: { enter_counts: Record<string, number> } };
      };
      return full.players[player].enter_counts?.[id] ?? 0;
    },
    [side, TEST_SUBJECT] as const,
  );
}

async function hoverHandCard(page: Page, card: string) {
  const el = page.locator(`#blueHand .card[data-card='${card}']`).first();
  await expect(el).toBeVisible({ timeout: 5000 });
  await el.hover();
  return el;
}

async function growEnterCount(page: Page, target: number) {
  const priority = [TEST_SUBJECT, HUMANE_LOVE, OBSIDIAN_RAVEN, SCHOLAR, SEPHIE];
  for (let i = 0; i < 120; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    const k = await enterCount(page);
    if (k >= target) return k;
    await resolveChoices(page);
    const snap = await page.evaluate((ids) => {
      const full = window.__arena!.full() as { active: string };
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const playId = ids.find((id) => legal.some((a) => a.play?.card === id));
      return { active: full.active, playId };
    }, priority);
    if (snap.active === "a" && snap.playId) {
      await playCard(page, snap.playId);
      continue;
    }
    await page.evaluate(() => {
      document.querySelector(".choice-modal")?.remove();
      const actingNow = document.getElementById("turnCounter")?.dataset.acting;
      const id = actingNow === "a" ? "endTurnBlue" : "endTurnRed";
      (document.getElementById(id) as HTMLButtonElement | null)?.click();
    });
    await page.waitForTimeout(200);
  }
  return enterCount(page);
}

function enterCountLine(k: number) {
  return `Obsessed Test Subjects entered: ${k}/5`;
}

test("test subject counters in tooltips track engine enter_counts", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  const deck = await importDeck(page, "meta-rune-test-subject.json", META_DECK);
  await startGame(page, deck);
  await confirmMulligans(page);
  await closeDrawer(page);

  let k = await enterCount(page);
  expect(k).toBe(0);

  for (const id of FOLLOWER_SUMMONERS) {
    const inHand = await page.evaluate((card) => {
      const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
        .players.a.hand;
      return hand.some((c) => c.card === card);
    }, id);
    if (!inHand) continue;
    const card = await hoverHandCard(page, id);
    const tooltip = page.locator("#cardTooltip");
    await expect(tooltip).toContainText(enterCountLine(k));
    if (id === SEPHIE) {
      await artShot(card, `${ART}/test_subject_sephie_hand_k0.png`);
    }
  }

  for (const id of NO_LINE_CARDS) {
    const inHand = await page.evaluate((card) => {
      const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
        .players.a.hand;
      return hand.some((c) => c.card === card);
    }, id);
    if (!inHand) continue;
    await hoverHandCard(page, id);
    await expect(page.locator("#cardTooltip")).not.toContainText("Obsessed Test Subjects entered:");
  }

  k = await growEnterCount(page, 5);
  expect(k, "enter_counts should reach 5 through real play").toBeGreaterThanOrEqual(5);

  for (const id of FOLLOWER_SUMMONERS) {
    const inHand = await page.evaluate((card) => {
      const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
        .players.a.hand;
      return hand.some((c) => c.card === card);
    }, id);
    if (!inHand) continue;
    const card = await hoverHandCard(page, id);
    const tooltip = page.locator("#cardTooltip");
    await expect(tooltip).toContainText(enterCountLine(k));
    if (k >= 5) {
      await expect(tooltip.locator(".dynamic-counter-line.gate-met")).toBeVisible();
    }
    if (id === SEPHIE) {
      await artShot(card, `${ART}/test_subject_sephie_hand_k5.png`);
    }
  }

  for (const id of NO_LINE_CARDS) {
    const inHand = await page.evaluate((card) => {
      const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
        .players.a.hand;
      return hand.some((c) => c.card === card);
    }, id);
    if (!inHand) continue;
    const el = await hoverHandCard(page, id);
    await expect(page.locator("#cardTooltip")).not.toContainText("Obsessed Test Subjects entered:");
    if (id === TEST_SUBJECT) {
      await expect(el).not.toHaveClass(/enhance-ready/);
    }
  }

  const oppK = await enterCount(page, "b");
  expect(oppK).not.toBe(k);
});
