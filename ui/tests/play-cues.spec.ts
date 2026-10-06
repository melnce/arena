import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const PAD = "10001110";
const FOLLOWER_B = "10011110";
const HARK = "10753310";
const KOU = "10411110";
const WILD_PROFUSION = "10011210";
const FAIRY = "90011110";
const ELD_AXE = "10671310";

function mixedDeck(cards: Record<string, number>): Record<string, number> {
  const total = Object.values(cards).reduce((a, b) => a + b, 0);
  const out = { ...cards };
  if (total < 40) out[PAD] = (out[PAD] ?? 0) + (40 - total);
  return out;
}

async function boot(page: Page, pace: string = "normal") {
  await page.addInitScript((p) => {
    localStorage.setItem("svwb.playCues", p);
  }, pace);
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
  opts: {
    mode?: string;
    seed?: string;
    first?: string;
    deckA: string;
    deckB?: string;
    human?: string;
    botPolicy?: string;
    policyA?: string;
    policyB?: string;
  },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode ?? "hotseat");
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  await page.locator("#redDeckSelect").selectOption(opts.deckB ?? opts.deckA);
  if (opts.human) await page.locator("#humanSideSelect").selectOption(opts.human);
  if (opts.botPolicy) {
    await page.locator("#vsBotPolicy").selectOption(opts.botPolicy);
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

async function mulliganKeep(page: Page, keep: string | string[]) {
  const cards = Array.isArray(keep) ? keep : [keep];
  await page.evaluate((keepCards) => {
    const legal = window.__arena!.legal() as Array<{
      mulligan?: { player: string; swap: boolean[] };
    }>;
    const mull = legal.find((a) => a.mulligan);
    if (!mull?.mulligan) return;
    const who = mull.mulligan.player as "a" | "b";
    const hand = (
      window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> }; b: { hand: Array<{ card: string }> } } }
    ).players[who].hand;
    const swap = hand.map((c) => !keepCards.includes(c.card));
    const act =
      legal.find(
        (a) =>
          a.mulligan?.player === who &&
          a.mulligan.swap.length === swap.length &&
          a.mulligan.swap.every((bit, idx) => bit === swap[idx]),
      ) ?? mull;
    window.__arena!.apply(act);
  }, cards);
}

async function confirmMulligans(page: Page, keepA?: string | string[], keepB?: string | string[]) {
  for (let i = 0; i < 2; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase !== "mulligan") break;
    const acting = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ mulligan?: { player: string } }>;
      return legal.find((a) => a.mulligan)?.mulligan?.player ?? null;
    });
    if (acting === "a" && keepA) await mulliganKeep(page, keepA);
    else if (acting === "b" && keepB) await mulliganKeep(page, keepB);
    else {
      const btn = page.locator(".mulligan-confirm-btn").locator("visible=true");
      if (await btn.count()) await btn.first().click();
    }
    await page.waitForTimeout(80);
  }
}

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function applyFirst(page: Page, key: string) {
  const ok = await page.evaluate((k) => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => k in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, key);
  expect(ok, `expected legal ${key}`).toBeTruthy();
}

async function playCard(page: Page, card: string, player?: "a" | "b") {
  if (player) await waitForActing(page, player);
  const ok = await page.evaluate((id) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === id);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, card);
  expect(ok, `expected play ${card}`).toBeTruthy();
}

async function endTurn(page: Page) {
  await page.evaluate(() => {
    const acting = document.getElementById("turnCounter")?.dataset.acting;
    const id = acting === "a" ? "endTurnBlue" : "endTurnRed";
    (document.getElementById(id) as HTMLButtonElement | null)?.click();
  });
}

async function waitForActing(page: Page, player: "a" | "b", timeout = 20_000) {
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", player, { timeout });
}

async function skipToPp(page: Page, player: "a" | "b", pp: number) {
  for (let i = 0; i < 30; i++) {
    const ready = await page.evaluate(
      ({ p, need }) => {
        const full = window.__arena!.full() as {
          active: string;
          players: { a: { pp: number }; b: { pp: number } };
        };
        return full.active === p && full.players[p].pp >= need;
      },
      { p: player, need: pp },
    );
    if (ready) return;
    await applyFirst(page, "end_turn");
  }
}

async function opponentFieldFollowers(page: Page, ...cards: string[]) {
  for (const card of cards) {
    await skipToPp(page, "b", 2);
    await playCard(page, card, "b");
    await applyFirst(page, "end_turn");
  }
  await waitForActing(page, "a");
}

function lastCueLog(page: Page) {
  return page.evaluate(() => {
    const log = window.__arena!.cueLog() ?? [];
    return log.length ? log[log.length - 1]! : null;
  });
}

async function waitCueArrow(page: Page, why: string) {
  const arrow = page.locator(`#cueLayer path[data-why="${why}"]`);
  await expect(arrow).toHaveCount(1, { timeout: 5000 });
  return arrow;
}

test.describe("play cues", () => {
  test("1 off mode: no overlay cues and zero-duration log", async ({ page }) => {
    await boot(page, "off");
    const deck = await importDeck(page, "cues-off.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "7", deckA: deck });
    await confirmMulligans(page);
    await closeDrawer(page);
    await skipToPp(page, "a", 2);
    await playCard(page, PAD, "a");
    const entry = await lastCueLog(page);
    expect(entry?.duration).toBe(0);
    expect(entry?.plan.cues.length).toBe(0);
    await expect(page.locator("#cueLayer path")).toHaveCount(0);
    await expect(page.locator("#cueLayer .cue-spotlight")).toHaveCount(0);
    await artShot(page.locator("#cueLayer"), `${ART}/cues_off.png`);
  });

  test("2 attack arrow with data attributes", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-atk.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "11", deckA: deck });
    await confirmMulligans(page, [PAD], [PAD]);
    await closeDrawer(page);
    await skipToPp(page, "a", 2);
    await playCard(page, PAD, "a");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "b", 2);
    await playCard(page, PAD, "b");
    await applyFirst(page, "end_turn");
    await waitForActing(page, "a");
    const slot = await page.evaluate((cardId) => {
      const full = window.__arena!.full() as {
        players: { a: { field: Array<{ card: string } | null> } };
      };
      return full.players.a.field.findIndex((c) => c?.card === cardId);
    }, PAD);
    expect(slot).toBeGreaterThanOrEqual(0);
    await page.evaluate(
      ({ slot }) => {
        window.__arena!.apply({
          attack: { player: "a", attacker_slot: slot, target: "leader" },
        });
      },
      { slot },
    );
    const arrow = await waitCueArrow(page, "attack");
    await expect(arrow).toHaveAttribute("data-style", "attack");
    await expect(arrow).toHaveAttribute("data-from", /slot:a:/);
    await expect(arrow).toHaveAttribute("data-to", "leader:b");
    const attrs = await arrow.evaluate((el) => ({
      x1: el.dataset.x1,
      y1: el.dataset.y1,
      x2: el.dataset.x2,
      y2: el.dataset.y2,
    }));
    expect(Number(attrs.x2)).toBeGreaterThan(0);
    expect(Number(attrs.y2)).toBeGreaterThan(0);
    await artShot(page, `${ART}/cues_attack.png`, { fullPage: true });
  });

  test("3 effect arrows from spell damage (Hark)", async ({ page }) => {
    await boot(page, "normal");
    const deckA = await importDeck(page, "cues-hark-a.json", mixedDeck({ [HARK]: 40 }));
    const deckB = await importDeck(
      page,
      "cues-hark-b.json",
      mixedDeck({ [PAD]: 20, [FOLLOWER_B]: 20 }),
    );
    await startGame(page, { seed: "43", deckA, deckB });
    await confirmMulligans(page, [HARK], [FOLLOWER_B]);
    await closeDrawer(page);
    await opponentFieldFollowers(page, PAD, FOLLOWER_B);
    await skipToPp(page, "a", 3);
    for (let i = 0; i < 20; i++) {
      const has = await page.evaluate(
        (id) => {
          const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
            .players.a.hand;
          return hand.some((c) => c.card === id);
        },
        HARK,
      );
      if (has) break;
      await applyFirst(page, "end_turn");
      await applyFirst(page, "end_turn");
    }
    await playCard(page, HARK, "a");
    const arrows = page.locator('#cueLayer path[data-why="effect"]');
    await expect(arrows.first()).toBeVisible({ timeout: 5000 });
    const entry = await lastCueLog(page);
    expect(entry?.plan.cues.some((c) => (c.cue as { kind?: string }).kind === "effect" || (c.cue as { type?: string }).type === "arrow")).toBeTruthy();
    await artShot(page, `${ART}/cues_effect.png`, { fullPage: true });
  });

  test("4 choice arrow from targeted spell", async ({ page }) => {
    await boot(page, "normal");
    const deckA = await importDeck(page, "cues-choice-a.json", mixedDeck({ [ELD_AXE]: 10, [PAD]: 30 }));
    const deckB = await importDeck(page, "cues-choice-b.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "56", deckA, deckB });
    await confirmMulligans(page, [ELD_AXE], [PAD]);
    await closeDrawer(page);
    await opponentFieldFollowers(page, PAD);
    await skipToPp(page, "a", 2);
    await playCard(page, ELD_AXE, "a");
    await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ choose?: { option: unknown } }>;
      const act = legal.find((a) => a.choose);
      if (act) window.__arena!.apply(act);
    });
    const arrow = await waitCueArrow(page, "choice");
    await expect(arrow).toHaveAttribute("data-style", "choice");
    await artShot(page, `${ART}/cues_choice.png`, { fullPage: true });
  });

  test("5 random arrow from amulet trigger", async ({ page }) => {
    await boot(page, "normal");
    const deckA = await importDeck(
      page,
      "cues-random-a.json",
      mixedDeck({ [WILD_PROFUSION]: 5, [FAIRY]: 5, [PAD]: 30 }),
    );
    const deckB = await importDeck(
      page,
      "cues-random-b.json",
      mixedDeck({ [PAD]: 20, [FOLLOWER_B]: 20 }),
    );
    await startGame(page, { seed: "44", deckA, deckB });
    await confirmMulligans(page, [WILD_PROFUSION, FAIRY], [FOLLOWER_B]);
    await closeDrawer(page);
    await opponentFieldFollowers(page, PAD, FOLLOWER_B);
    await skipToPp(page, "a", 3);
    await playCard(page, WILD_PROFUSION, "a");
    await playCard(page, FAIRY, "a");
    const arrow = await waitCueArrow(page, "random");
    await expect(arrow).toHaveAttribute("data-style", "random");
    await artShot(page, `${ART}/cues_random.png`, { fullPage: true });
  });

  test("6 fade cue on destroy", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-destroy.json", mixedDeck({ [WILD_PROFUSION]: 40 }));
    await startGame(page, { seed: "59", deckA: deck });
    await confirmMulligans(page, [WILD_PROFUSION]);
    await closeDrawer(page);
    await skipToPp(page, "b", 2);
    await playCard(page, WILD_PROFUSION, "b");
    for (let i = 0; i < 6; i++) await applyFirst(page, "end_turn");
    await expect(page.locator("#cueLayer .cue-fade")).toHaveCount(1, { timeout: 8000 });
    const hasFade = await page.evaluate(() =>
      (window.__arena!.cueLog() ?? []).some((e) =>
        e.plan.cues.some((c) => (c.cue as { type?: string }).type === "fade"),
      ),
    );
    expect(hasFade).toBeTruthy();
    await artShot(page, `${ART}/cues_fade.png`, { fullPage: true });
  });

  test("7 spotlight on resolve source", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-spot.json", mixedDeck({ [KOU]: 10, [PAD]: 30 }));
    await startGame(page, { seed: "60", deckA: deck });
    await confirmMulligans(page, [KOU], [PAD]);
    await closeDrawer(page);
    await skipToPp(page, "a", 2);
    await playCard(page, PAD, "a");
    await skipToPp(page, "a", 7);
    await playCard(page, KOU, "a");
    await applyFirst(page, "end_turn");
    await applyFirst(page, "end_turn");
    await waitForActing(page, "a");
    const kouSlot = await page.evaluate((cardId) => {
      const full = window.__arena!.full() as {
        players: { a: { field: Array<{ card: string } | null> } };
      };
      return full.players.a.field.findIndex((c) => c?.card === cardId);
    }, KOU);
    expect(kouSlot).toBeGreaterThanOrEqual(0);
    await page.evaluate(({ slot }) => {
      window.__arena!.apply({
        attack: { player: "a", attacker_slot: slot, target: "leader" },
      });
    }, { slot: kouSlot });
    await expect(page.locator("#cueLayer .cue-spotlight")).toHaveCount(1, { timeout: 5000 });
    await artShot(page, `${ART}/cues_spotlight.png`, { fullPage: true });
  });

  test("8 bot play spotlight when step is not human", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-bot.json", mixedDeck({ [PAD]: 40 }));
    await openSettings(page);
    await page.locator("#modeSelect").selectOption("vs-bot");
    await page.locator("#hideBotHandToggle").evaluate((el) => {
      (el as HTMLInputElement).checked = false;
    });
    await startGame(page, {
      mode: "vs-bot",
      seed: "3",
      deckA: deck,
      human: "a",
      botPolicy: "first-legal",
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    await expect
      .poll(
        async () =>
          page.evaluate(() => {
            return (window.__arena!.cueLog() ?? []).some(
              (e) =>
                !e.human &&
                e.plan.cues.some(
                  (c) =>
                    (c.cue as { type?: string; target?: { kind?: string } }).type === "spotlight" &&
                    (c.cue as { target?: { kind?: string } }).target?.kind === "hand",
                ),
            );
          }),
        { timeout: 30_000 },
      )
      .toBeTruthy();
    await artShot(page, `${ART}/cues_bot_play.png`, { fullPage: true });
  });

  test("9 pace timing: stagger 120ms and hold durations", async ({ page }) => {
    for (const [pace, hold] of [
      ["fast", 600],
      ["normal", 1000],
      ["slow", 1800],
    ] as const) {
      await boot(page, pace);
      const deck = await importDeck(page, `cues-pace-${pace}.json`, mixedDeck({ [PAD]: 40 }));
      await startGame(page, { seed: "5", deckA: deck });
      await confirmMulligans(page, [PAD], [PAD]);
      await closeDrawer(page);
      await skipToPp(page, "a", 2);
      await playCard(page, PAD, "a");
      await applyFirst(page, "end_turn");
      await skipToPp(page, "b", 2);
      await playCard(page, PAD, "b");
      await applyFirst(page, "end_turn");
      await waitForActing(page, "a");
      const slot = await page.evaluate((cardId) => {
        const full = window.__arena!.full() as {
          players: { a: { field: Array<{ card: string } | null> } };
        };
        return full.players.a.field.findIndex((c) => c?.card === cardId);
      }, PAD);
      await page.evaluate(
        ({ slot }) => {
          window.__arena!.apply({
            attack: { player: "a", attacker_slot: slot, target: "leader" },
          });
        },
        { slot },
      );
      const entry = await lastCueLog(page);
      const n = entry!.plan.cues.length;
      const expected = n > 0 ? (n - 1) * 120 + hold : 0;
      expect(entry?.duration).toBe(expected);
      const staggered = entry!.plan.cues;
      if (staggered.length > 1) {
        expect(staggered[1]!.at - staggered[0]!.at).toBe(120);
      }
    }
    await artShot(page.locator("body"), `${ART}/cues_pace.png`);
  });

  test("10 watch mode respects cue timing; hidden hand filters bot cues", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-watch.json", mixedDeck({ [PAD]: 40 }));
    await openSettings(page);
    await page.locator("#watchSpeed").evaluate((el) => {
      const input = el as HTMLInputElement;
      input.value = "10";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await page.locator("#watchAutoStart").evaluate((el) => {
      (el as HTMLInputElement).checked = false;
    });
    await startGame(page, {
      mode: "watch",
      seed: "2",
      deckA: deck,
      policyA: "random",
      policyB: "first-legal",
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    await expect(page.locator("#watchBar")).toBeVisible();
    const baseDelay = await page.evaluate(() => window.__arena!.watchDelayMs());
    expect(baseDelay).toBeLessThan(300);
    const h0 = await page.evaluate(() => window.__arena!.hash());
    await page.locator("#watchPlayBtn").click();
    await expect.poll(() => page.evaluate(() => window.__arena!.hash())).not.toBe(h0);
    await expect(page.locator("#cueLayer path")).toHaveCount(0);
    await expect
      .poll(
        () =>
          page.evaluate(
            () => (window.__arena!.cueLog() ?? []).some((e) => e.duration > 0),
          ),
        { timeout: 30_000 },
      )
      .toBeTruthy();
    await page.locator("#watchPauseBtn").click();

    await openSettings(page);
    await page.locator("#modeSelect").selectOption("vs-bot");
    await page.locator("#hideBotHandToggle").check();
    await page.locator("#humanSideSelect").selectOption("a");
    await page.locator("#vsBotPolicy").selectOption("first-legal");
    await page.locator("#policyBSelect").selectOption("first-legal", { force: true });
    await page.locator("#seedInput").fill("4");
    await page.locator("#startGameBtn").click();
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    const botEntry = await lastCueLog(page);
    const hidden = botEntry!.plan.cues.every((c) => {
      const cue = c.cue as { type?: string; target?: { kind?: string; player?: string } };
      if (cue.type === "spotlight" || cue.type === "fade") {
        return cue.target?.player !== "b" || cue.target?.kind !== "hand";
      }
      return true;
    });
    expect(hidden).toBeTruthy();
    await artShot(page, `${ART}/cues_watch_hidden.png`, { fullPage: true });
  });
});
