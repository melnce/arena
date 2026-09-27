import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

const CALGE = "10634120";
const DEPTHS = "90034330";
const CRYSTALSPAWN = "10631110";
const FILLER = "10001110";

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

async function startGame(
  page: Page,
  opts: { seed?: string; deckA: string; deckB: string },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  await page.locator("#redDeckSelect").selectOption(opts.deckB);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmMulligansKeepCalge(page: Page) {
  for (let i = 0; i < 2; i++) {
    await page.evaluate((calge) => {
      const legal = window.__arena!.legal() as Array<{
        mulligan?: { player: string; swap: boolean[] };
      }>;
      const mull = legal.find((a) => a.mulligan);
      if (!mull?.mulligan) return;
      const who = mull.mulligan.player;
      const hand = (
        window.__arena!.full() as {
          players: { a: { hand: Array<{ card: string }> }; b: { hand: Array<{ card: string }> } };
        }
      ).players[who as "a" | "b"].hand;
      const swap =
        who === "a"
          ? hand.map((c) => c.card !== calge)
          : hand.map(() => false);
      const act = legal.find(
        (a) =>
          a.mulligan?.player === who &&
          a.mulligan.swap.length === swap.length &&
          a.mulligan.swap.every((bit, idx) => bit === swap[idx]),
      );
      window.__arena!.apply(act ?? mull);
    }, CALGE);
    await expect(page.locator("#turnCounter")).not.toHaveAttribute("data-phase", "mulligan", {
      timeout: 5000,
    }).catch(() => undefined);
  }
}

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function driveToCalgeEvolved(page: Page) {
  for (let i = 0; i < 240; i++) {
    const step = await page.evaluate(({ calge }) => {
        const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
        if (legal.some((a) => "mulligan" in a)) return "mulligan";

        const choose = legal.find((a) => "choose" in a);
        if (choose) {
          window.__arena!.apply(choose);
          return "choose";
        }
        const confirm = legal.find((a) => "confirm" in a);
        if (confirm) {
          window.__arena!.apply(confirm);
          return "confirm";
        }

        const full = window.__arena!.full() as {
          active: string;
          players: {
            a: {
              hand: Array<{ card: string }>;
              field: Array<{ card: string } | null>;
            };
          };
        };

        const calgeSlot = full.players.a.field.findIndex((c) => c?.card === calge);
        if (calgeSlot >= 0 && window.__arena!.playerInfo("a").evolve_unlocked) {
          const evo = legal.find(
            (a) =>
              (a.evolve as { slot?: number; super?: boolean } | undefined)?.slot === calgeSlot &&
              !(a.evolve as { super?: boolean }).super,
          );
          if (evo) {
            window.__arena!.apply(evo);
            return "evolve";
          }
        }

        if (full.active === "a") {
          const info = window.__arena!.playerInfo("a");
          const playCalge = legal.find(
            (a) => (a.play as { card?: string } | undefined)?.card === calge,
          );
          if (info.evolve_unlocked && playCalge) {
            window.__arena!.apply(playCalge);
            return "play-calge";
          }
        }

        const end = legal.find((a) => "end_turn" in a);
        if (end) {
          window.__arena!.apply(end);
          return "end";
        }
        return `stuck:${JSON.stringify(legal.map((a) => Object.keys(a)[0]))}`;
      },
      { calge: CALGE },
    );
    if (step === "evolve") return;
    if (step === "mulligan") throw new Error("unexpected mulligan mid-drive");
    if (typeof step === "string" && step.startsWith("stuck:")) {
      const detail = await page.evaluate(() => {
        const full = window.__arena!.full() as {
          winner?: string | null;
          phase: string | Record<string, unknown>;
          active: string;
        };
        return {
          winner: full.winner ?? null,
          phase: full.phase,
          active: full.active,
        };
      });
      throw new Error(`drive stalled at step ${i}: ${step} state=${JSON.stringify(detail)}`);
    }
  }
  throw new Error("Calge evolve never happened");
}

test("Calge evolve adds Depths; playing it spends faith across board and leaders", async ({
  page,
}) => {
  test.setTimeout(180_000);
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(String(err)));

  await boot(page);
  const deck = await importDeck(page, "calge-depths.json", {
    [CALGE]: 3,
    [CRYSTALSPAWN]: 37,
  });
  const fodder = await importDeck(page, "fodder.json", { [FILLER]: 40 });
  await startGame(page, { seed: "0", deckA: deck, deckB: fodder });
  await confirmMulligansKeepCalge(page);
  await closeDrawer(page);

  const afterMull = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      turn: number;
      players: { a: { hand: Array<{ card: string }>; pp: number } };
    };
    return {
      turn: full.turn,
      pp: full.players.a.pp,
      hand: full.players.a.hand.map((c) => c.card),
    };
  });
  expect(afterMull.hand).toContain(CALGE);

  await driveToCalgeEvolved(page);

  const toast = page.locator(".toast-error, .error-toast, [data-toast-kind='error']");
  await expect(toast).toHaveCount(0, { timeout: 1000 });

  const depthsInHand = await page.evaluate((depths) => {
    const full = window.__arena!.full() as {
      players: { a: { hand: Array<{ card: string }> } };
    };
    return full.players.a.hand.some((c) => c.card === depths);
  }, DEPTHS);
  expect(depthsInHand).toBeTruthy();

  const faithBefore = await page.evaluate(() => {
    const full = window.__arena!.full() as { players: { a: { faith: number } } };
    return full.players.a.faith;
  });

  let played = false;
  for (let i = 0; i < 40; i++) {
    played = await page.evaluate((depths) => {
      const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
      const choose = legal.find((a) => "choose" in a);
      if (choose) {
        window.__arena!.apply(choose);
        return false;
      }
      const confirm = legal.find((a) => "confirm" in a);
      if (confirm) {
        window.__arena!.apply(confirm);
        return false;
      }
      const act = legal.find(
        (a) => (a.play as { card?: string } | undefined)?.card === depths,
      );
      if (act) {
        window.__arena!.apply(act);
        return true;
      }
      const end = legal.find((a) => "end_turn" in a);
      if (end) {
        window.__arena!.apply(end);
        return false;
      }
      return false;
    }, DEPTHS);
    if (played) break;
  }
  expect(played, "Depths play action").toBeTruthy();
  await expect(toast).toHaveCount(0, { timeout: 1000 });

  const outcome = await page.evaluate(
    ({ faithBefore, spawn }) => {
      const full = window.__arena!.full() as {
        players: {
          a: {
            faith: number;
            leader_defense: number;
            field: Array<{ card: string; attack: number; defense: number } | null>;
          };
          b: { leader_defense: number };
        };
      };
      const spawns = full.players.a.field.filter((c) => c?.card === spawn);
      const newest = spawns[spawns.length - 1];
      const buff = newest ? newest.attack - 1 : 0;
      const leaderGain = full.players.a.leader_defense - 20;
      const enemyLoss = 20 - full.players.b.leader_defense;
      return {
        faithBefore,
        faithAfter: full.players.a.faith,
        buff,
        leaderGain: Math.max(0, leaderGain),
        enemyLoss: Math.max(0, enemyLoss),
        spawn: newest,
      };
    },
    { faithBefore, spawn: CRYSTALSPAWN },
  );

  expect(outcome.spawn).toBeTruthy();
  expect(outcome.buff + outcome.leaderGain + outcome.enemyLoss).toBe(faithBefore);
  expect(outcome.faithAfter).toBeGreaterThanOrEqual(faithBefore);
  expect(errors).toEqual([]);
});
