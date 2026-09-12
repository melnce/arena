import { expect, test, type Locator, type Page } from "@playwright/test";
import { ART, artShot, assertGlow, assertPngLeftEdge } from "./helpers.ts";

const RUSH = "10631110";
const RUSH_2_2 = "10621110";
const FIGHTER = "10001110";
const LEAH = "10001120";
const TIKOH = "10463110";

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

async function importDeck(page: Page, name: string, cards: Record<string, number>) {
  await openSettings(page);
  await page.locator("#deckImportFileInput").setInputFiles({
    name,
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(cards)),
  });
  return `import-${name.replace(/\.json$/i, "")}`;
}

async function startGame(page: Page, deckId: string, deckB?: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckId);
  await page.locator("#redDeckSelect").selectOption(deckB ?? deckId);
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

async function playCard(page: Page, card: string) {
  const ok = await page.evaluate((id) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === id);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, card);
  expect(ok, `expected play ${card}`).toBeTruthy();
}

test("evolved-this-turn follower glows yellow only, then green next turn", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "fighter.json", { [FIGHTER]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });

  for (let i = 0; i < 24; i++) {
    const snap = await page.evaluate((cardId) => {
      const full = window.__arena!.full() as {
        active: "a" | "b";
        players: { a: { field: unknown[] }; b: { field: unknown[] } };
      };
      const legal = window.__arena!.legal() as Array<{
        play?: { card: string };
        evolve?: { super: boolean };
        end_turn?: unknown;
      }>;
      return {
        active: full.active,
        evo: window.__arena!.playerInfo("a").evolve_unlocked,
        enemy: full.players.b.field.filter(Boolean).length,
        own: full.players.a.field.filter(Boolean).length,
        canPlay: legal.some((a) => a.play?.card === cardId),
        canEvo: legal.some((a) => a.evolve && !a.evolve.super),
        canEnd: legal.some((a) => a.end_turn),
      };
    }, FIGHTER);
    if (snap.active === "a" && snap.evo && snap.enemy >= 1 && snap.own === 0 && snap.canPlay) {
      await playCard(page, FIGHTER);
      break;
    }
    if (snap.active === "b" && snap.enemy < 1 && snap.canPlay) {
      await playCard(page, FIGHTER);
      continue;
    }
    if (snap.canEnd) {
      await applyFirst(page, "end_turn");
      continue;
    }
    throw new Error(`stuck driving to evolve-on-play (i=${i})`);
  }

  const evo = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
    return legal.find((a) => a.evolve && !a.evolve.super) ?? null;
  });
  expect(evo).toBeTruthy();
  await page.evaluate((act) => window.__arena!.apply(act), evo);

  const card = page.locator("#blueBoard .card[data-card='10001110']").first();
  await expect(card).toHaveClass(/evolved/);
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");
  const outline = await card.locator(".card-image-wrapper").evaluate((el) => getComputedStyle(el).outlineColor);
  expect(outline).toBe("rgb(255, 212, 0)");
  const yellowPath = await artShot(card, `${ART}/entry_evo_yellow.png`);
  assertPngLeftEdge(yellowPath, "yellow");

  await applyFirst(page, "end_turn");
  await expect(card).not.toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "none");

  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "green");
  const greenPath = await artShot(card, `${ART}/entry_evo_green.png`);
  assertPngLeftEdge(greenPath, "green");
});

test("rush follower is yellow, then no glow on the opponent's turn, then green", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "rush.json", { [RUSH]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });

  // Enemy follower first so the Rush card has a legal attack (not the leader).
  for (let i = 0; i < 8; i++) {
    const snap = await page.evaluate((cardId) => {
      const full = window.__arena!.full() as { active: "a" | "b" };
      const legal = window.__arena!.legal() as Array<{
        play?: { card: string };
        end_turn?: unknown;
      }>;
      return {
        active: full.active,
        canPlay: legal.some((a) => a.play?.card === cardId),
        canEnd: legal.some((a) => a.end_turn),
      };
    }, RUSH);
    if (snap.active === "b" && snap.canPlay) {
      await playCard(page, RUSH);
      await applyFirst(page, "end_turn");
      break;
    }
    if (snap.canEnd) {
      await applyFirst(page, "end_turn");
      continue;
    }
    throw new Error(`stuck seeding enemy rush (i=${i})`);
  }

  await playCard(page, RUSH);
  const card = page.locator("#blueBoard .card[data-card='10631110']").first();
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");
  await artShot(card, `${ART}/rush_glow_entry_yellow.png`);
  const entryReason = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ cannot_attack_reason?: string | null }>;
    return info[0]?.cannot_attack_reason ?? null;
  });
  expect(entryReason).toBe("Cannot attack the leader: it entered the field this turn");
  await card.hover();
  const entryTip = page.locator("#cardTooltip .tooltip-leader-blocked");
  await expect(entryTip).toHaveText("Cannot attack the leader: it entered the field this turn");

  await applyFirst(page, "end_turn");
  await expect(card).not.toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "none");
  await artShot(card, `${ART}/rush_glow_opponent_none.png`);

  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "green");
  await artShot(card, `${ART}/rush_glow_next_turn_green.png`);
});

test("rush follower on an empty enemy board has no glow", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "rush-empty.json", { [RUSH]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
  await playCard(page, RUSH);
  const card = page.locator("#blueBoard .card[data-card='10631110']").first();
  await expect(card).toBeVisible();
  await expect(card).not.toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "none");
  await artShot(card, `${ART}/rush_glow_empty_board_none.png`);
});

test("rush follower loses glow after it attacks", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const me = await importDeck(page, "rush-22.json", { [RUSH_2_2]: 40 });
  const them = await importDeck(page, "rush-11.json", { [RUSH]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });

  await applyFirst(page, "end_turn");
  await playCard(page, RUSH);
  await applyFirst(page, "end_turn");
  await playCard(page, RUSH_2_2);
  const card = page.locator("#blueBoard .card[data-card='10621110']").first();
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");

  const hit = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ attack?: { player: string } }>;
    const act = legal.find((a) => a.attack?.player === "a");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(hit, "expected a legal attack").toBeTruthy();
  await expect(card).toBeVisible();
  await expect(card).not.toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "none");
  await artShot(card, `${ART}/rush_glow_after_attack_none.png`);
});

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function dragOnto(page: Page, src: Locator, dst: Locator) {
  const a = await src.boundingBox();
  const b = await dst.boundingBox();
  expect(a, "drag source box").toBeTruthy();
  expect(b, "drop target box").toBeTruthy();
  await page.mouse.move(a!.x + a!.width / 2, a!.y + a!.height / 2);
  await page.mouse.down();
  await page.mouse.move(b!.x + b!.width / 2, b!.y + b!.height / 2, { steps: 12 });
}

async function fieldFollower(
  page: Page,
  player: "a" | "b",
  card: string,
): Promise<{ slot: number; attacksLeft: number; uid: number }> {
  return page.evaluate(
    ({ player: who, card: id }) => {
      const full = window.__arena!.full() as {
        players: Record<
          string,
          { field: Array<{ card: string; id: number; flags: { attacks_left: number } } | null> }
        >;
      };
      const i = full.players[who].field.findIndex((f) => f?.card === id);
      const inst = full.players[who].field[i];
      if (i < 0 || !inst) throw new Error(`no ${id} on ${who}`);
      return { slot: i, attacksLeft: inst.flags.attacks_left, uid: inst.id };
    },
    { player, card },
  );
}

test("enemy Ward is yellow, leader drop is refused, killing Ward turns green", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const me = await importDeck(page, "ward-atk.json", { [RUSH_2_2]: 40 });
  const them = await importDeck(page, "ward-leah.json", { [LEAH]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await playCard(page, RUSH_2_2);
  await applyFirst(page, "end_turn");
  await playCard(page, LEAH);
  await applyFirst(page, "end_turn");

  const card = page.locator("#blueBoard .card[data-card='10621110']").first();
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");
  const outline = await card.locator(".card-image-wrapper").evaluate((el) => getComputedStyle(el).outlineColor);
  expect(outline).toBe("rgb(255, 212, 0)");
  const yellowPath = await artShot(card, `${ART}/leader_glow_ward_yellow.png`);
  assertPngLeftEdge(yellowPath, "yellow");

  const wardReason = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ cannot_attack_reason?: string | null }>;
    return info[0]?.cannot_attack_reason ?? null;
  });
  expect(wardReason).toBe("Cannot attack the leader: an enemy Ward is in play");
  await card.hover();
  await expect(page.locator("#cardTooltip .tooltip-leader-blocked")).toHaveText(
    "Cannot attack the leader: an enemy Ward is in play",
  );

  const before = await fieldFollower(page, "a", RUSH_2_2);
  const leader = page.locator("#redLeader");
  await dragOnto(page, card, leader);
  const highlighted = await leader.evaluate((el) => el.classList.contains("pointer-drop-highlight"));
  expect(highlighted).toBe(false);
  await page.mouse.up();
  const afterRefuse = await fieldFollower(page, "a", RUSH_2_2);
  expect(afterRefuse.attacksLeft).toBe(before.attacksLeft);
  await expect(card).toHaveClass(/rush-glow/);

  await playCard(page, RUSH_2_2);
  const killed = await page.evaluate((keepSlot) => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { player: string; attacker_slot: number; target: unknown };
    }>;
    const act = legal.find(
      (a) => a.attack?.player === "a" && a.attack.attacker_slot !== keepSlot && a.attack.target !== "leader",
    );
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, before.slot);
  expect(killed, "expected the new Rush follower to kill Ward").toBeTruthy();

  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "green");
  const greenPath = await artShot(card, `${ART}/leader_glow_ward_cleared_green.png`);
  assertPngLeftEdge(greenPath, "green");
});

test("printed can't-attack-leader follower is yellow every turn", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const me = await importDeck(page, "printed-lock-a.json", { [FIGHTER]: 40 });
  const them = await importDeck(page, "printed-lock-b.json", { [FIGHTER]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await playCard(page, FIGHTER);
  await applyFirst(page, "end_turn");
  await playCard(page, FIGHTER);
  await applyFirst(page, "end_turn");

  const card = page.locator("#blueBoard .card[data-card='10001110']").first();
  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);

  const slot = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ slot: number }>;
    return info[0]?.slot ?? 0;
  });
  await page.evaluate((s) => window.__arena!.debugGrantCantAttackLeader("a", s), slot);
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");
  const outline = await card.locator(".card-image-wrapper").evaluate((el) => getComputedStyle(el).outlineColor);
  expect(outline).toBe("rgb(255, 212, 0)");
  const yellowPath = await artShot(card, `${ART}/leader_glow_printed_yellow.png`);
  assertPngLeftEdge(yellowPath, "yellow");

  const printed = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ cannot_attack_reason?: string | null }>;
    return info[0]?.cannot_attack_reason ?? null;
  });
  expect(printed).toBe("Cannot attack the leader: printed restriction");
  await card.hover();
  await expect(page.locator("#cardTooltip .tooltip-leader-blocked")).toHaveText(
    "Cannot attack the leader: printed restriction",
  );
  await expect(card.locator(".cant_attack-overlay")).toHaveCount(0);

  await applyFirst(page, "end_turn");
  await expect(card).not.toHaveClass(/rush-glow/);
  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/rush-glow/);
  await expect(card).not.toHaveClass(/can-attack/);
  await assertGlow(card, "yellow");
});

test("0-attack follower is green and a leader drop applies", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const me = await importDeck(page, "zero-atk.json", { [TIKOH]: 40 });
  const them = await importDeck(page, "zero-atk-opp.json", { [FIGHTER]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

  await playCard(page, TIKOH);
  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");

  const card = page.locator("#blueBoard .card[data-card='10463110']").first();
  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "green");
  const outline = await card.locator(".card-image-wrapper").evaluate((el) => getComputedStyle(el).outlineColor);
  expect(outline).toBe("rgb(57, 217, 138)");
  const greenPath = await artShot(card, `${ART}/leader_glow_zero_atk_green.png`);
  assertPngLeftEdge(greenPath, "green");

  const before = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: { field: Array<{ flags: { attacks_left: number } } | null> };
        b: { leader_defense: number };
      };
    };
    const inst = full.players.a.field.find((f) => f) ?? null;
    return { def: full.players.b.leader_defense, attacksLeft: inst?.flags.attacks_left ?? -1 };
  });
  expect(before.attacksLeft).toBeGreaterThan(0);

  const leader = page.locator("#redLeader");
  await dragOnto(page, card, leader);
  const highlighted = await leader.evaluate((el) => el.classList.contains("pointer-drop-highlight"));
  expect(highlighted).toBe(true);
  await page.mouse.up();

  const after = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: { field: Array<{ flags: { attacks_left: number } } | null> };
        b: { leader_defense: number };
      };
    };
    const inst = full.players.a.field.find((f) => f) ?? null;
    return { def: full.players.b.leader_defense, attacksLeft: inst?.flags.attacks_left ?? -1 };
  });
  expect(after.def).toBe(before.def);
  expect(after.attacksLeft).toBe(0);
  await expect(card).not.toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "none");
  await artShot(card, `${ART}/leader_glow_zero_atk_after.png`);
});
