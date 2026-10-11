import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { ART, artShot, openSettings } from "./helpers.ts";

const EARRINGS = "10761210";
const HAVEN_AMULET_DECK = JSON.parse(
  readFileSync(new URL("../../oracle/decks/meta-haven-amulet.json", import.meta.url), "utf8"),
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

async function startGame(page: Page, deckId: string, seed = "67") {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill(seed);
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

async function driveEarringsEngageWithHandPick(page: Page): Promise<boolean> {
  return page.evaluate((earrings) => {
    const arena = window.__arena!;
    const legal = () => arena.legal() as Array<Record<string, unknown>>;
    const applyFirst = (pred: (a: Record<string, unknown>) => boolean) => {
      const act = legal().find(pred);
      if (!act) return false;
      arena.apply(act);
      return true;
    };
    const full = () =>
      arena.full() as {
        active: "a" | "b";
        phase: string | { choice?: unknown };
        players: { a: { hand: Array<{ card: string }>; field: Array<{ card?: string } | null> } };
      };
    const endTurn = () => applyFirst((a) => "end_turn" in a);
    const earringsSlot = () =>
      full().players.a.field.findIndex((c) => c?.card === earrings);
    for (let guard = 0; guard < 400; guard++) {
      const st = full();
      if (typeof st.phase === "object") {
        applyFirst((a) => "choose" in a);
        continue;
      }
      if (st.active === "a") {
        const slot = earringsSlot();
        if (slot >= 0) {
          const engaged = applyFirst(
            (a) => (a.engage as { slot?: number } | undefined)?.slot === slot,
          );
          if (engaged) {
            const st2 = full();
            const earringsGone = earringsSlot() < 0;
            const handPick = legal().some((a) => {
              const opt = (a as { choose?: { option?: { card?: string } } }).choose?.option;
              return typeof opt?.card === "string";
            });
            if (earringsGone && handPick && typeof st2.phase === "object") return true;
          }
        }
        applyFirst((a) => "play" in a);
        applyFirst((a) => "attack" in a);
        endTurn();
        continue;
      }
      applyFirst((a) => "play" in a);
      applyFirst((a) => "attack" in a);
      endTurn();
    }
    return false;
  }, EARRINGS);
}

type HandCard = { card: string; id: number };

test("earrings engage replicates fanfare after destroy", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  const deck = await importDeck(page, "meta-haven-amulet.json", HAVEN_AMULET_DECK);
  await startGame(page, deck, "67");
  await confirmMulligans(page);
  await closeDrawer(page);

  await expect
    .poll(async () => driveEarringsEngageWithHandPick(page), {
      timeout: 120_000,
      intervals: [500, 1000, 2000],
    })
    .toBe(true);

  const prompt = page.locator(".choice-prompt-bar");
  await expect(prompt).toBeVisible();
  await expect(prompt).toContainText("Select a card in your hand");
  await artShot(prompt, `${ART}/earrings_engage_hand_pick.png`);

  const before = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: {
          hand: HandCard[];
          deck: HandCard[];
          field: Array<{ card?: string } | null>;
        };
      };
    };
    const legal = window.__arena!.legal() as Array<{
      choose?: { option?: { card?: string; inst?: number } };
    }>;
    const pick = legal.find((a) => {
      const opt = (a as { choose?: { option?: { card?: string } } }).choose?.option;
      return typeof opt?.card === "string";
    }) as { choose?: { option?: { card: string } } } | undefined;
    if (!pick?.choose?.option?.card) throw new Error("no hand pick option");
    const card = pick.choose.option.card;
    const deckIdsForCard = full.players.a.deck
      .filter((c) => c.card === card)
      .map((c) => c.id);
    const deckCount = deckIdsForCard.length;
    return {
      handLen: full.players.a.hand.length,
      deckLen: full.players.a.deck.length,
      pickedCard: card,
      deckInstanceIdsBefore: deckIdsForCard,
      deckCountBefore: deckCount,
    };
  });

  await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ choose?: unknown }>;
    const act = legal.find((a) => a.choose);
    if (!act) throw new Error("expected hand pick");
    window.__arena!.apply(act);
  });
  await expect(prompt).toBeHidden({ timeout: 5000 });

  const after = await page.evaluate(
    ({ pickedCard, deckInstanceIdsBefore }) => {
      const full = window.__arena!.full() as {
        players: {
          a: {
            hand: HandCard[];
            deck: HandCard[];
            field: Array<{ card?: string } | null>;
          };
        };
      };
      const deckForCard = full.players.a.deck.filter((c) => c.card === pickedCard);
      const beforeSet = new Set(deckInstanceIdsBefore);
      const newInstances = deckForCard.filter((c) => !beforeSet.has(c.id)).map((c) => c.id);
      return {
        handLen: full.players.a.hand.length,
        deckLen: full.players.a.deck.length,
        deckCount: deckForCard.length,
        newInstanceId: newInstances.length === 1 ? newInstances[0] : null,
        earringsOnField: full.players.a.field.some((c) => c?.card === "10761210"),
      };
    },
    {
      pickedCard: before.pickedCard,
      deckInstanceIdsBefore: before.deckInstanceIdsBefore,
    },
  );

  expect(after.handLen).toBe(before.handLen);
  expect(after.deckLen).toBe(before.deckLen);
  expect(after.deckCount).toBe(before.deckCountBefore + 1);
  expect(after.newInstanceId).not.toBeNull();
  expect(after.earringsOnField).toBe(false);
  await artShot(page.locator("#blueBoard"), `${ART}/earrings_engage_after_pick.png`);
});
