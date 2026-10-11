import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { ART, artShot, openSettings } from "./helpers.ts";

const KUKISHIRO = "10564120";
const FOX = "10061120";
const FALCON = "90061110";
const ODD_COSTS = [1, 3, 5];
const EVEN_COSTS = [2, 4, 6];
const SEED = "97";

const KUKISHIRO_DECK = JSON.parse(
  readFileSync(new URL("../../oracle/decks/meta-haven-kukishiro.json", import.meta.url), "utf8"),
) as Record<string, number>;

type WasmEvent =
  | { draw: { player: string; card: string } }
  | { summon: { player: string; card: string; slot: number } }
  | Record<string, unknown>;

type FanfareOutcome = {
  events: WasmEvent[];
  freeSlotsBeforeA: number;
  freeSlotsB: number;
  hasCrest: boolean;
  hasKukishiro: boolean;
};

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

async function startGame(page: Page, deckId: string, seed = SEED) {
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

async function driveAndPlayKukishiroFanfare(page: Page): Promise<FanfareOutcome | null> {
  return page.evaluate(({ kukishiro }) => {
    const arena = window.__arena!;
    const legal = () => arena.legal() as Array<Record<string, unknown>>;
    const parseEvents = (raw: unknown): WasmEvent[] => {
      if (typeof raw === "string") return JSON.parse(raw) as WasmEvent[];
      if (Array.isArray(raw)) return raw as WasmEvent[];
      return [];
    };
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
        players: {
          a: {
            pp: number;
            hand: Array<{ card: string }>;
            field: Array<unknown | null>;
            crests: Array<{ id: string }>;
          };
          b: { field: Array<unknown | null> };
        };
      };
    const emptySlots = (field: Array<unknown | null>) => field.filter((c) => c == null).length;
    const endTurn = () => applyFirst((a) => "end_turn" in a);

    for (let guard = 0; guard < 500; guard++) {
      const st = full();
      if (typeof st.phase === "object") {
        applyFirst((a) => "choose" in a);
        continue;
      }
      if (st.active === "a") {
        const handIdx = st.players.a.hand.findIndex((c) => c.card === kukishiro);
        if (handIdx >= 0 && st.players.a.pp >= 7) {
          const freeA = emptySlots(st.players.a.field);
          const freeB = emptySlots(st.players.b.field);
          if (freeA >= 1) {
            const events: WasmEvent[] = [];
            const play = legal().find((a) => {
              const p = a.play as { hand_pos?: number; hand?: number } | undefined;
              const pos = p?.hand_pos ?? p?.hand;
              return pos === handIdx;
            });
            if (!play) return null;
            events.push(...parseEvents(arena.apply(play)));
            for (let c = 0; c < 40; c++) {
              const mid = full();
              if (typeof mid.phase !== "object") break;
              const choose = legal().find((a) => "choose" in a);
              if (!choose) break;
              events.push(...parseEvents(arena.apply(choose)));
            }
            const fin = full();
            return {
              events,
              freeSlotsBeforeA: freeA,
              freeSlotsB: freeB,
              hasCrest: fin.players.a.crests.some((c) => c.id === "crest:10564120"),
              hasKukishiro: fin.players.a.field.some((c) => (c as { card?: string })?.card === kukishiro),
            };
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
    return null;
  }, { kukishiro: KUKISHIRO });
}

function isTokenSummon(e: WasmEvent): { player: string; card: string } | null {
  if (!("summon" in e) || !e.summon) return null;
  const { player, card } = e.summon as { player: string; card: string };
  if (card !== FOX && card !== FALCON) return null;
  return { player, card };
}

test("kukishiro fanfare crest summons fox or falcon on draws", async ({ page }) => {
  test.setTimeout(240_000);
  await boot(page);
  const deck = await importDeck(page, "meta-haven-kukishiro.json", KUKISHIRO_DECK);
  await startGame(page, deck, SEED);
  await confirmMulligans(page);
  await closeDrawer(page);

  const fanfare = await driveAndPlayKukishiroFanfare(page);
  expect(fanfare).not.toBeNull();
  const { events, freeSlotsBeforeA, freeSlotsB, hasCrest, hasKukishiro } = fanfare!;

  expect(hasCrest).toBe(true);
  expect(hasKukishiro).toBe(true);

  const draws = events.filter(
    (e): e is { draw: { player: string; card: string } } =>
      typeof e === "object" && e !== null && "draw" in e && (e.draw as { player: string }).player === "a",
  );
  expect(draws).toHaveLength(2);

  const drawCosts: number[] = [];
  for (const d of draws) {
    const cost = await page.evaluate((cardId) => {
      const info = window.__arena!.cardText(cardId) as { cost?: number | null };
      if (info.cost == null) throw new Error(`no cost for ${cardId}`);
      return info.cost;
    }, d.draw.card);
    drawCosts.push(cost);
  }
  const oddDraws = drawCosts.filter((c) => ODD_COSTS.includes(c)).length;
  const evenDraws = drawCosts.filter((c) => EVEN_COSTS.includes(c)).length;

  const summonsA = events.filter((e) => isTokenSummon(e)?.player === "a").length;
  const summonsB = events.filter((e) => isTokenSummon(e)?.player === "b").length;

  const freeSlotsAfterK = freeSlotsBeforeA - 1;
  const expectedA = Math.min(oddDraws, freeSlotsAfterK);
  const expectedB = Math.min(evenDraws, freeSlotsB);

  expect(freeSlotsAfterK).toBeGreaterThan(0);
  expect(oddDraws).toBeGreaterThan(0);
  expect(expectedA).toBeGreaterThan(0);

  expect(summonsA).toBe(expectedA);
  expect(summonsB).toBe(expectedB);

  await artShot(page.locator("#blueBoard"), `${ART}/kukishiro_crest_summons.png`);
});
