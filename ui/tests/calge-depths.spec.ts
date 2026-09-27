import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

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

async function confirmMulligans(page: Page) {
  for (let i = 0; i < 2; i++) {
    const btn = page.locator(".mulligan-confirm-btn").locator("visible=true");
    if (await btn.count()) {
      await btn.first().click();
      await expect(btn).toBeHidden({ timeout: 5000 }).catch(() => undefined);
    }
  }
}

async function applyAction(page: Page, action: unknown) {
  await page.evaluate((act) => window.__arena!.apply(act), action);
}

async function skipToEvolve(page: Page) {
  for (let i = 0; i < 40; i++) {
    const info = await page.evaluate(() => window.__arena!.playerInfo("a"));
    if (info.evolve_unlocked) return;
    await applyAction(page, { end_turn: { player: "a" } });
    await applyAction(page, { end_turn: { player: "b" } });
  }
  throw new Error("evolve never unlocked");
}

test("Calge evolve adds Depths; playing it spends faith across board and leaders", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(String(err)));

  await boot(page);
  const deck = await importDeck(page, "calge-depths.json", {
    "10634120": 10,
    "10631110": 10,
    "88001110": 20,
  });
  await startGame(page, { seed: "4242", deckA: deck, deckB: deck });
  await confirmMulligans(page);

  await skipToEvolve(page);

  const calgeSlot = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      play?: { card: string };
      evolve?: { slot: number; super: boolean };
    }>;
    const play = legal.find((a) => a.play?.card === "10634120");
    if (play?.play) {
      window.__arena!.apply(play);
      return null;
    }
    const evo = legal.find((a) => a.evolve && !a.evolve.super);
    if (!evo?.evolve) return -1;
    window.__arena!.apply(evo);
    return evo.evolve.slot;
  });
  expect(calgeSlot, "Calge evolve action").not.toBe(-1);

  const toast = page.locator(".toast-error, .error-toast, [data-toast-kind='error']");
  await expect(toast).toHaveCount(0, { timeout: 1000 });

  const depthsInHand = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { hand: Array<{ card: string }> } };
    };
    return full.players.a.hand.some((c) => c.card === "90034330");
  });
  expect(depthsInHand).toBeTruthy();

  const faithBefore = await page.evaluate(() => {
    const full = window.__arena!.full() as { players: { a: { faith: number } } };
    return full.players.a.faith;
  });

  const played = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === "90034330");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(played, "Depths play action").toBeTruthy();
  await expect(toast).toHaveCount(0, { timeout: 1000 });

  const outcome = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: {
          faith: number;
          leader: { defense: number };
          field: Array<{ card: string; attack: number; defense: number } | null>;
        };
        b: { leader: { defense: number } };
      };
    };
    const spawns = full.players.a.field.filter((c) => c?.card === "10631110");
    const newest = spawns[spawns.length - 1];
    const buff = newest ? newest.attack - 1 : 0;
    const leaderGain = full.players.a.leader.defense - 20;
    const enemyLoss = 20 - full.players.b.leader.defense;
    return {
      faithBefore,
      faithAfter: full.players.a.faith,
      buff,
      leaderGain: Math.max(0, leaderGain),
      enemyLoss: Math.max(0, enemyLoss),
      spawn: newest,
    };
  });

  expect(outcome.spawn).toBeTruthy();
  expect(outcome.buff + outcome.leaderGain + outcome.enemyLoss).toBe(faithBefore);
  expect(outcome.faithAfter).toBeGreaterThanOrEqual(faithBefore);
  expect(errors).toEqual([]);
});
