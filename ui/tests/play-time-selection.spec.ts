import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

const FIGHTER = "10001110";
const MOELLE = "10811130";
const WORLD = "10503210";

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
  await page.locator("#seedInput").fill("901");
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

type Snap = {
  phase: string;
  pending: string | null;
  wogCd: number | null;
  handCards: string[];
  chooseHandCards: string[];
};

async function snap(page: Page): Promise<Snap> {
  return page.evaluate(() => {
    const full = window.__arena!.full() as {
      phase: string | { choice?: { node?: { targets?: { pending?: string; options?: unknown[] } } } };
      players: { a: { hand: Array<{ card: string }>; field: Array<{ card?: string; countdown?: number | null } | null> } };
    };
    const phaseObj = full.phase;
    const phase = typeof phaseObj === "string" ? phaseObj : "choice";
    const pending =
      typeof phaseObj === "object"
        ? (phaseObj.choice?.node as { targets?: { pending?: string } } | undefined)?.targets?.pending ??
          null
        : null;
    const wog = full.players.a.field.find((c) => c?.card === "10503210");
    const legal = window.__arena!.legal() as Array<{ choose?: { option?: { card?: string } } }>;
    const chooseHandCards = legal
      .filter((a) => a.choose?.option?.card)
      .map((a) => a.choose!.option!.card!);
    return {
      phase,
      pending,
      wogCd: wog?.countdown ?? null,
      handCards: full.players.a.hand.map((c) => c.card),
      chooseHandCards,
    };
  });
}

async function driveToMoelleWog(page: Page): Promise<void> {
  const ok = await page.evaluate(
    ({ fighter, moelle, world }) => {
      const arena = window.__arena!;
      const ids = { fighter, moelle, world };

      const full = () =>
        arena.full() as {
          active: "a" | "b";
          phase: string | { choice?: unknown };
          players: {
            a: {
              pp: number;
              hand: Array<{ card: string }>;
              field: Array<{ card?: string; countdown?: number | null } | null>;
            };
          };
        };

      const legal = () => arena.legal() as Array<Record<string, unknown>>;

      const applyFirst = (pred: (a: Record<string, unknown>) => boolean) => {
        const act = legal().find(pred);
        if (!act) return false;
        arena.apply(act);
        return true;
      };

      const play = (card: string) =>
        applyFirst((a) => (a.play as { card?: string } | undefined)?.card === card);

      const endTurn = () => applyFirst((a) => "end_turn" in a);

      const chooseHand = (card: string) =>
        applyFirst((a) => (a.choose as { option?: { card?: string } } | undefined)?.option?.card === card);

      const wogCd = () => {
        const w = full().players.a.field.find((c) => c?.card === ids.world);
        return w?.countdown ?? null;
      };

      const hasMoelle = () => full().players.a.hand.some((c) => c.card === ids.moelle);
      const hasKeeper = () => full().players.a.hand.some((c) => c.card === ids.fighter);

      // Mulligans done; give ourselves PP and a clean setup via repeated turns.
      for (let guard = 0; guard < 200; guard++) {
        const st = full();
        if (typeof st.phase === "object") {
          const choose = legal().find((a) => "choose" in a);
          if (choose) arena.apply(choose);
          continue;
        }
        if (st.active !== "a") {
          endTurn();
          continue;
        }
        const cd = wogCd();
        const wogOnBoard = cd != null;
        if (wogOnBoard && cd === 1 && hasMoelle() && hasKeeper() && st.players.a.pp >= 1) {
          return true;
        }
        if (!wogOnBoard && st.players.a.pp >= 1 && play(ids.world)) continue;
        if (wogOnBoard && cd != null && cd > 1 && st.players.a.pp >= 1) {
          if (!full().players.a.field.some((c) => c?.card === ids.fighter)) {
            if (play(ids.fighter)) continue;
          } else if (play(ids.fighter)) continue;
        }
        if (hasMoelle() && !hasKeeper() && play(ids.fighter)) continue;
        endTurn();
      }
      return false;
    },
    { fighter: FIGHTER, moelle: MOELLE, world: WORLD },
  );
  expect(ok, "failed to reach Moelle + World of Games at count 1").toBeTruthy();
}

test("Moelle + World of Games: play-time pick before Last Words draws", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const deck = {
    [WORLD]: 10,
    [MOELLE]: 10,
    [FIGHTER]: 20,
  };
  const id = await importDeck(page, "moelle-wog.json", deck);
  await startGame(page, id);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });

  await driveToMoelleWog(page);
  const before = await snap(page);
  expect(before.wogCd).toBe(1);
  expect(before.handCards).toContain(MOELLE);
  expect(before.handCards).toContain(FIGHTER);
  const preHand = [...before.handCards];

  const played = await page.evaluate((moelle) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === moelle);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, MOELLE);
  expect(played).toBeTruthy();

  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  await expect(page.locator(".choice-prompt-bar")).toBeVisible();

  const afterPlay = await snap(page);
  expect(afterPlay.pending).toBe("play_select");
  expect(afterPlay.wogCd).toBe(1);
  expect(afterPlay.handCards.length).toBe(preHand.length - 1);
  expect(afterPlay.chooseHandCards.length).toBeGreaterThan(0);
  for (const id of afterPlay.chooseHandCards) {
    expect(preHand).toContain(id);
  }
  const freshDraws = afterPlay.chooseHandCards.filter((id) => !preHand.includes(id));
  expect(freshDraws, "World of Games draws are not candidates yet").toEqual([]);
});
