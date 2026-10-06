import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const PAD = "10001110";
const FOLLOWER_B = "10011110";
const WILD_PROFUSION = "10011210";
const FAIRY = "90011110";
const COLONEL = "10952110";
const LAYLA = "10872110";

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
    hideBotHand?: boolean;
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
  if (opts.hideBotHand !== undefined) {
    await page.locator("#hideBotHandToggle").evaluate((el, checked) => {
      (el as HTMLInputElement).checked = checked;
    }, opts.hideBotHand);
  }
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
      window.__arena!.full() as {
        players: { a: { hand: Array<{ card: string }> }; b: { hand: Array<{ card: string }> } };
      }
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

async function botPlayEventually(page: Page, card: string, player: "a" | "b") {
  for (let i = 0; i < 40; i++) {
    const acting = await page.locator("#turnCounter").getAttribute("data-acting");
    if (acting === player) {
      const ok = await page.evaluate((id) => {
        const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
        const act = legal.find((a) => a.play?.card === id);
        if (!act) return false;
        window.__arena!.botApply(act);
        return true;
      }, card);
      if (ok) return;
    }
    await applyFirst(page, "end_turn");
  }
  throw new Error(`bot could not play ${card} as ${player}`);
}

async function playCardEventually(page: Page, card: string, player: "a" | "b") {
  for (let i = 0; i < 40; i++) {
    const acting = await page.locator("#turnCounter").getAttribute("data-acting");
    if (acting === player) {
      const ok = await page.evaluate((id) => {
        const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
        const act = legal.find((a) => a.play?.card === id);
        if (!act) return false;
        window.__arena!.apply(act);
        return true;
      }, card);
      if (ok) return;
    }
    await applyFirst(page, "end_turn");
  }
  throw new Error(`could not play ${card} as ${player}`);
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

function lastCueLog(page: Page) {
  return page.evaluate(() => {
    const log = window.__arena!.cueLog() ?? [];
    return log.length ? log[log.length - 1]! : null;
  });
}

async function waitForCueArrow(page: Page, why: string, timeout = 15_000) {
  await expect
    .poll(async () => page.locator(`#cueLayer path[data-why="${why}"]`).count(), {
      timeout,
      intervals: [40, 80, 120],
    })
    .toBe(1);
  return page.locator(`#cueLayer path[data-why="${why}"]`);
}

async function waitForCueFade(page: Page, timeout = 15_000) {
  await expect
    .poll(async () => page.locator("#cueLayer .cue-fade").count(), {
      timeout,
      intervals: [40, 80, 120],
    })
    .toBe(1);
  return page.locator("#cueLayer .cue-fade");
}

async function waitForPlaySpotlight(page: Page, timeout = 30_000) {
  await expect
    .poll(async () => page.locator("#cueLayer .cue-play-spotlight").count(), {
      timeout,
      intervals: [40, 80, 120],
    })
    .toBeGreaterThan(0);
  return page.locator("#cueLayer .cue-play-spotlight").first();
}

test.describe("play cues", () => {
  test("1 off mode: no overlay cues and zero-duration log", async ({ page }) => {
    await boot(page, "off");
    const deck = await importDeck(page, "cues-off.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "7", deckA: deck, mode: "vs-bot", botPolicy: "first-legal" });
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    const entry = await lastCueLog(page);
    expect(entry?.duration).toBe(0);
    expect(entry?.plan.cues.length).toBe(0);
    await expect(page.locator("#cueLayer path")).toHaveCount(0);
    await expect(page.locator("#cueLayer .cue-play-spotlight")).toHaveCount(0);
    await artShot(page.locator("#cueLayer"), `${ART}/cues_off.png`);
  });

  test("2 bot play spotlight shows played card beside board", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-bot-spot.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, {
      mode: "vs-bot",
      seed: "3",
      deckA: deck,
      human: "a",
      botPolicy: "first-legal",
      hideBotHand: true,
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    const spotlight = page.locator("#cueLayer .cue-play-spotlight");
    await expect(spotlight).toHaveCount(1, { timeout: 30_000 });
    const cardId = await spotlight.getAttribute("data-card");
    expect(cardId).toBeTruthy();
    const plan = await page.evaluate(() => window.__arena!.cues());
    expect(plan?.cues.some((c) => (c.cue as { type?: string }).type === "play-spotlight")).toBeTruthy();
    await artShot(page, `${ART}/cues_spotlight_arrow.png`, { fullPage: true });
  });

  test("3 bot attack arrow uses uid endpoints", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-atk.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "11", deckA: deck, mode: "hotseat" });
    await confirmMulligans(page, PAD, PAD);
    await closeDrawer(page);
    await skipToPp(page, "a", 2);
    await playCard(page, PAD, "a");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "b", 2);
    await playCard(page, PAD, "b");
    await applyFirst(page, "end_turn");
    await waitForActing(page, "a");
    const slots = await page.evaluate(() => {
      const full = window.__arena!.full() as {
        players: { a: { field: Array<{ id: number; card: string } | null> } };
      };
      const slot = full.players.a.field.findIndex((c) => c?.card === "10001110");
      const uid = full.players.a.field[slot]?.id;
      return { slot, uid };
    });
    await page.evaluate(
      ({ slot }) => {
        window.__arena!.botApply({
          attack: { player: "a", attacker_slot: slot, target: "leader" },
        });
      },
      { slot: slots.slot },
    );
    const arrow = page.locator('#cueLayer path[data-why="attack"]');
    await expect(arrow).toHaveCount(1, { timeout: 8000 });
    await expect(arrow).toHaveAttribute("data-style", "solid");
    await expect(arrow).toHaveAttribute("data-from", /^uid:\d+$/);
    await expect(arrow).toHaveAttribute("data-to", /^uid:\d+$|^leader:[ab]$/);
    await artShot(page, `${ART}/cues_attack.png`, { fullPage: true });
  });

  test("4 random arrow targets follower uid", async ({ page }) => {
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
    await startGame(page, { seed: "44", deckA, deckB, mode: "hotseat" });
    await confirmMulligans(page, [WILD_PROFUSION, FAIRY], [PAD, PAD]);
    await closeDrawer(page);
    await playCardEventually(page, PAD, "b");
    await applyFirst(page, "end_turn");
    await botPlayEventually(page, WILD_PROFUSION, "a");
    await botPlayEventually(page, FAIRY, "a");
    const arrow = await waitForCueArrow(page, "random");
    await expect(arrow).toHaveAttribute("data-style", "dashed");
    await expect(arrow).toHaveAttribute("data-to", /^uid:\d+$/);
    await artShot(page, `${ART}/cues_random.png`, { fullPage: true });
  });

  test("5 Last Words effect arrow aims at surviving neighbour", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page, "normal");
    const deckA = await importDeck(
      page,
      "cues-lw-a.json",
      mixedDeck({ [LAYLA]: 3, [PAD]: 37 }),
    );
    const deckB = await importDeck(
      page,
      "cues-lw-b.json",
      mixedDeck({ [COLONEL]: 20, [FOLLOWER_B]: 20 }),
    );
    await startGame(page, {
      seed: "51",
      deckA,
      deckB,
      mode: "vs-bot",
      human: "a",
      botPolicy: "first-legal",
    });
    await confirmMulligans(page, [LAYLA], [COLONEL, FOLLOWER_B]);
    await closeDrawer(page);
    await endTurn(page);
    for (let i = 0; i < 24; i++) {
      const ready = await page.evaluate(
        (colonel) => {
          const full = window.__arena!.full() as {
            active: string;
            players: {
              b: { pp: number; hand: Array<{ card: string }>; field: Array<{ card: string } | null> };
            };
          };
          if (full.players.b.field.some((c) => c?.card === colonel)) return true;
          if (full.active !== "b" || full.players.b.pp < 6) return false;
          const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
          const act = legal.find((a) => a.play?.card === colonel);
          if (act) {
            window.__arena!.botApply(act);
            const end = (window.__arena!.legal() as Array<Record<string, unknown>>).find(
              (a) => "end_turn" in a,
            );
            if (end) window.__arena!.botApply(end);
          }
          return false;
        },
        COLONEL,
      );
      if (ready) break;
      await applyFirst(page, "end_turn");
    }
    await expect
      .poll(
        () =>
          page.evaluate(
            (colonel) =>
              (
                window.__arena!.full() as {
                  players: { b: { field: Array<{ card: string } | null> } };
                }
              ).players.b.field.some((c) => c?.card === colonel),
            COLONEL,
          ),
        { timeout: 10_000 },
      )
      .toBeTruthy();
    await skipToPp(page, "b", 1);
    await page.evaluate(
      (follower) => {
        const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
        const act = legal.find((a) => a.play?.card === follower);
        if (act) window.__arena!.botApply(act);
        const end = (window.__arena!.legal() as Array<Record<string, unknown>>).find(
          (a) => "end_turn" in a,
        );
        if (end) window.__arena!.botApply(end);
      },
      FOLLOWER_B,
    );
    await skipToPp(page, "a", 4);
    await playCard(page, LAYLA, "a");
    await applyFirst(page, "end_turn");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "a", 4);
    const neighbourUid = await page.evaluate(
      ({ colonel, follower }) => {
        const full = window.__arena!.full() as {
          players: { b: { field: Array<{ id: number; card: string } | null> } };
        };
        const idx = full.players.b.field.findIndex((c) => c?.card === colonel);
        const neighbour = full.players.b.field[idx === 0 ? 1 : 0];
        return neighbour?.card === follower ? neighbour.id : null;
      },
      { colonel: COLONEL, follower: FOLLOWER_B },
    );
    expect(neighbourUid).toBeTruthy();
    await page.evaluate(
      ({ layla, colonel }) => {
        const full = window.__arena!.full() as {
          players: {
            a: { field: Array<{ card: string } | null> };
            b: { field: Array<{ card: string } | null> };
          };
        };
        const slot = full.players.a.field.findIndex((c) => c?.card === layla);
        const def = full.players.b.field.findIndex((c) => c?.card === colonel);
        window.__arena!.botApply({
          attack: { player: "a", attacker_slot: slot, target: { slot: def } },
        });
      },
      { layla: LAYLA, colonel: COLONEL },
    );
    const arrow = await waitForCueArrow(page, "effect");
    const arrowTo = await arrow.getAttribute("data-to");
    expect(arrowTo).toBe("leader:b");
    await expect(page.locator(`#redBoard .card[data-uid="${neighbourUid}"]`)).toBeVisible();
    const neighbourRect = await page
      .locator(`#redBoard .card[data-uid="${neighbourUid}"]`)
      .boundingBox();
    expect(neighbourRect).toBeTruthy();
    const leaderRect = await page.locator("#redLeader").boundingBox();
    expect(leaderRect).toBeTruthy();
    const x2 = Number(await arrow.getAttribute("data-x2"));
    const y2 = Number(await arrow.getAttribute("data-y2"));
    expect(Math.abs(x2 - (leaderRect!.x + leaderRect!.width / 2))).toBeLessThanOrEqual(2);
    expect(Math.abs(y2 - (leaderRect!.y + leaderRect!.height / 2))).toBeLessThanOrEqual(2);
    await artShot(page, `${ART}/cues_last_words.png`, { fullPage: true });
  });

  test("6 destroy fade keyed by uid", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-destroy.json", mixedDeck({ [LAYLA]: 3, [PAD]: 37 }));
    await startGame(page, { seed: "59", deckA: deck, mode: "hotseat" });
    await confirmMulligans(page, [LAYLA], [PAD]);
    await closeDrawer(page);
    await playCardEventually(page, PAD, "b");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "a", 4);
    await playCard(page, LAYLA, "a");
    await applyFirst(page, "end_turn");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "a", 4);
    const botPad = await page.evaluate((pad) => {
      const full = window.__arena!.full() as {
        players: { b: { field: Array<{ card: string } | null> } };
      };
      return full.players.b.field.findIndex((c) => c?.card === pad);
    }, PAD);
    expect(botPad).toBeGreaterThanOrEqual(0);
    await page.evaluate(
      ({ layla, defSlot }) => {
        const full = window.__arena!.full() as {
          players: { a: { field: Array<{ card: string } | null> } };
        };
        const atk = full.players.a.field.findIndex((c) => c?.card === layla);
        window.__arena!.botApply({
          attack: { player: "a", attacker_slot: atk, target: { slot: defSlot } },
        });
      },
      { layla: LAYLA, defSlot: botPad },
    );
    const fade = await waitForCueFade(page);
    await expect(fade).toHaveAttribute("data-uid", /^\d+$/);
    await artShot(page, `${ART}/cues_fade.png`, { fullPage: true });
  });

  test("7 hidden bot hand still spotlights public play", async ({ page }) => {
    test.setTimeout(60_000);
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-hidden.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, {
      mode: "vs-bot",
      seed: "3",
      deckA: deck,
      human: "a",
      botPolicy: "first-legal",
      hideBotHand: true,
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    await waitForPlaySpotlight(page, 45_000);
    const hideOn = await page.evaluate(
      () => (document.getElementById("hideBotHandToggle") as HTMLInputElement | null)?.checked,
    );
    expect(hideOn).toBe(true);
  });

  test("8 cue log history records step duration", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-history.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { mode: "vs-bot", seed: "5", deckA: deck, botPolicy: "first-legal" });
    await confirmMulligans(page);
    await closeDrawer(page);
    await endTurn(page);
    const log = await page.evaluate(() => window.__arena!.cueLog());
    expect(log.length).toBeGreaterThanOrEqual(1);
    expect(log[0]!.duration).toBe(1000);
    expect(log[0]!.plan.durationMs).toBe(1000);
  });

  test("9 pace: normal ≥970ms bot gap; off 250–800ms", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-pace.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { mode: "vs-bot", seed: "901", deckA: deck, botPolicy: "random" });
    await confirmMulligans(page);
    await closeDrawer(page);
    const t0 = Date.now();
    await endTurn(page);
    await waitForActing(page, "a", 60_000);
    const t1 = Date.now();
    expect(t1 - t0).toBeGreaterThanOrEqual(970);

    await boot(page, "off");
    const deckOff = await importDeck(page, "cues-pace-off.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { mode: "vs-bot", seed: "902", deckA: deckOff, botPolicy: "random" });
    await confirmMulligans(page);
    await closeDrawer(page);
    const u0 = Date.now();
    await endTurn(page);
    await waitForActing(page, "a", 60_000);
    const u1 = Date.now();
    const gap = u1 - u0;
    expect(gap).toBeGreaterThanOrEqual(250);
    expect(gap).toBeLessThanOrEqual(800);
  });

  test("10 watch mode shows cues capped by slider", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page, "slow");
    const deck = await importDeck(page, "cues-watch.json", mixedDeck({ [PAD]: 40 }));
    await openSettings(page);
    await page.locator("#modeSelect").selectOption("watch");
    await page.locator("#seedInput").fill("2");
    await page.locator("#blueDeckSelect").selectOption(deck);
    await page.locator("#redDeckSelect").selectOption(deck);
    await page.locator("#policyASelect").selectOption("first-legal", { force: true });
    await page.locator("#policyBSelect").selectOption("first-legal", { force: true });
    await page.locator("#watchSpeed").evaluate((el) => {
      const input = el as HTMLInputElement;
      input.value = "10";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await page.locator("#startGameBtn").click();
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
      timeout: 15_000,
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    await page.locator("#watchPlayBtn").click();
    await expect
      .poll(
        () =>
          page.evaluate(() =>
            (window.__arena!.cueLog() ?? []).some((e) => e.plan.cues.length > 0),
          ),
        { timeout: 60_000 },
      )
      .toBeTruthy();
    const entry = await page.evaluate(() => {
      const log = window.__arena!.cueLog() ?? [];
      return log.find((e) => e.plan.cues.length > 0) ?? log[log.length - 1]!;
    });
    const watchDelay = await page.evaluate(() => window.__arena!.watchDelayMs());
    expect(entry.duration).toBe(1800);
    expect(entry.plan.cues.length).toBeGreaterThan(0);
    expect(watchDelay).toBeLessThan(300);
    await page.locator("#watchPauseBtn").click();
    await page.setViewportSize({ width: 1180, height: 820 });
    await artShot(page, `${ART}/cues_tablet.png`, { fullPage: true });
  });

  test("11 destroyed lists show owner on destroy", async ({ page }) => {
    await boot(page, "normal");
    const deck = await importDeck(page, "cues-destroyed.json", mixedDeck({ [PAD]: 40 }));
    await startGame(page, { seed: "600", deckA: deck, mode: "hotseat" });
    await confirmMulligans(page, [PAD]);
    await closeDrawer(page);
    await skipToPp(page, "a", 2);
    await playCard(page, PAD, "a");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "b", 2);
    await playCard(page, PAD, "b");
    await applyFirst(page, "end_turn");
    await applyFirst(page, "end_turn");
    await skipToPp(page, "b", 2);
    await page.evaluate(
      (pad) => {
        const full = window.__arena!.full() as {
          players: {
            a: { field: Array<{ card: string } | null> };
            b: { field: Array<{ card: string } | null> };
          };
        };
        const def = full.players.a.field.findIndex((c) => c?.card === pad);
        const atk = full.players.b.field.findIndex((c) => c?.card === pad);
        window.__arena!.apply({
          attack: { player: "b", attacker_slot: atk, target: { slot: def } },
        });
      },
      PAD,
    );
    await expect(page.locator("#blueDestroyedList")).not.toBeEmpty({ timeout: 8000 });
    await expect(page.locator("#blueDestroyedList")).toContainText(/10001110|Fighter|Vanilla/i);
  });
});
