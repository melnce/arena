import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const TIES = "10922310";
const QUICK = "10021110";
const REAPER = "10953310";

type PlayerId = "a" | "b";
type TargetOpt = { slot: number; player: PlayerId };

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
  const id = `import-${name.replace(/\.json$/i, "")}`;
  await expect(page.locator("#blueDeckSelect")).toHaveValue(id, { timeout: 10_000 });
  return id;
}

async function startGame(
  page: Page,
  opts: { first: "a" | "b"; deckA: string; deckB: string },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption(opts.first);
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

type Full = {
  active: PlayerId;
  phase: { choice?: { player: PlayerId; node: { targets?: { options: TargetOpt[] } } } } | string;
  players: {
    a: { field: Array<{ card?: string; defense?: number } | null>; pp: number };
    b: { field: Array<{ card?: string; defense?: number } | null>; pp: number };
  };
};

async function snapshot(page: Page) {
  return page.evaluate(() => {
    const full = window.__arena!.full() as Full;
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    return { full, legal };
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

function occupied(field: Full["players"]["a"]["field"]) {
  return field.filter(Boolean).length;
}

async function driveToChoice(
  page: Page,
  opts: {
    caster: PlayerId;
    spell: string;
    ownNeed: number;
    enemyNeed: number;
    ownPlay: string;
    enemyPlay: string;
  },
) {
  const enemy: PlayerId = opts.caster === "a" ? "b" : "a";
  for (let i = 0; i < 24; i++) {
    const { full, legal } = await snapshot(page);
    if (typeof full.phase === "object" && full.phase.choice) {
      const hasSpell = legal.some((a) => "choose" in a);
      if (hasSpell) return;
    }
    const active = full.active;
    const ownN = occupied(full.players[opts.caster].field);
    const enemyN = occupied(full.players[enemy].field);
    const canPlay = (id: string) => legal.some((a) => (a.play as { card?: string } | undefined)?.card === id);
    if (active === opts.caster && canPlay(opts.spell) && ownN >= opts.ownNeed && enemyN >= opts.enemyNeed) {
      await playCard(page, opts.spell);
      return;
    }
    if (active === opts.caster && ownN < opts.ownNeed && canPlay(opts.ownPlay)) {
      await playCard(page, opts.ownPlay);
      continue;
    }
    if (active === enemy && enemyN < opts.enemyNeed && canPlay(opts.enemyPlay)) {
      await playCard(page, opts.enemyPlay);
      continue;
    }
    if (legal.some((a) => "end_turn" in a)) {
      await applyFirst(page, "end_turn");
      continue;
    }
    throw new Error(`stuck driving to ${opts.spell} choice (turn ${i})`);
  }
  throw new Error(`never reached ${opts.spell} choice`);
}

async function highlightedBoard(page: Page) {
  return page.evaluate(() => {
    const cards = [...document.querySelectorAll<HTMLElement>(".board-zone .card.legal-target, .board-zone .card.selectable")];
    return cards.map((el) => ({
      player: el.dataset.player,
      slot: Number(el.dataset.slot),
      board: el.closest(".board-zone")?.id ?? "",
    }));
  });
}

async function assertHighlightsMatchNode(page: Page, poolOwner: PlayerId) {
  const nodeOpts = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      phase: { choice?: { node: { targets?: { options: TargetOpt[] } } } };
    };
    return full.phase.choice?.node.targets?.options ?? [];
  });
  expect(nodeOpts.length).toBeGreaterThan(0);
  expect(nodeOpts.every((o) => o.player === poolOwner)).toBeTruthy();

  const marks = await highlightedBoard(page);
  const markedKeys = new Set(marks.map((m) => `${m.player}:${m.slot}`));
  const nodeKeys = new Set(nodeOpts.map((o) => `${o.player}:${o.slot}`));
  expect(markedKeys).toEqual(nodeKeys);

  const other: PlayerId = poolOwner === "a" ? "b" : "a";
  const otherBoard = other === "a" ? "blueBoard" : "redBoard";
  expect(marks.filter((m) => m.board === otherBoard || m.player === other)).toHaveLength(0);
}

async function bootSwordChoice(page: Page, first: "a" | "b") {
  await boot(page);
  const sword = await importDeck(page, `sword-${first}.json`, { [QUICK]: 20, [TIES]: 20 });
  const fodder = await importDeck(page, `fodder-${first}.json`, { [QUICK]: 40 });
  const deckA = first === "b" ? fodder : sword;
  const deckB = first === "b" ? sword : fodder;
  await startGame(page, { first, deckA, deckB });
  await confirmMulligans(page);
  await closeDrawer(page);
}

test("Severed Ties highlights the enemy board when red acts from the top", async ({ page }) => {
  await bootSwordChoice(page, "b");
  await driveToChoice(page, {
    caster: "b",
    spell: TIES,
    ownNeed: 1,
    enemyNeed: 2,
    ownPlay: QUICK,
    enemyPlay: QUICK,
  });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  await expect(page.locator("body")).toHaveClass(/select-mode/);

  const before = await page.evaluate(() => {
    const full = window.__arena!.full() as Full;
    const opts = (full.phase as { choice: { node: { targets: { options: TargetOpt[] } } } }).choice
      .node.targets.options;
    return {
      opts,
      def: full.players.a.field[1]?.defense ?? null,
      card: full.players.a.field[1]?.card ?? null,
    };
  });
  expect(before.opts.map((o) => o.player).every((p) => p === "a")).toBeTruthy();
  expect(before.opts.some((o) => o.slot === 1)).toBeTruthy();

  await assertHighlightsMatchNode(page, "a");
  await expect(page.locator("#redBoard .card.legal-target, #redBoard .card.selectable")).toHaveCount(0);
  await expect(page.locator("#blueBoard .card.legal-target")).toHaveCount(before.opts.length);

  await artShot(page.locator("#appRoot"), `${ART}/choice_red_acting_enemy_highlights.png`);

  await page.locator('#blueBoard .card[data-slot="1"]').click();
  await expect(page.locator("#turnCounter")).not.toHaveAttribute("data-phase", "choice", {
    timeout: 10_000,
  });

  const after = await page.evaluate(() => {
    const full = window.__arena!.full() as Full;
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const actions = window.__arena!.actions() as Array<{
      choose?: { option?: { slot?: number; player?: string } };
    }>;
    return {
      hasChoose: legal.some((a) => "choose" in a),
      field: full.players.a.field,
      lastChoose: [...actions].reverse().find((a) => a.choose)?.choose?.option ?? null,
    };
  });
  expect(after.hasChoose).toBeFalsy();
  expect(after.lastChoose).toEqual({ slot: 1, player: "a" });
  const survivor = after.field[1];
  if (survivor && before.def != null) {
    expect(survivor.defense).toBe(before.def - 5);
  } else {
    expect(survivor == null || survivor.defense === 0).toBeTruthy();
  }
});

test("Severed Ties highlights the enemy board when blue acts from the bottom", async ({ page }) => {
  await bootSwordChoice(page, "a");
  await driveToChoice(page, {
    caster: "a",
    spell: TIES,
    ownNeed: 1,
    enemyNeed: 2,
    ownPlay: QUICK,
    enemyPlay: QUICK,
  });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  await assertHighlightsMatchNode(page, "b");
  await expect(page.locator("#blueBoard .card.legal-target, #blueBoard .card.selectable")).toHaveCount(0);

  const beforeDef = await page.evaluate(() => {
    const full = window.__arena!.full() as Full;
    return full.players.b.field[1]?.defense ?? null;
  });

  await page.locator('#redBoard .card[data-slot="1"]').click();
  await expect(page.locator("#turnCounter")).not.toHaveAttribute("data-phase", "choice", {
    timeout: 10_000,
  });

  const after = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const actions = window.__arena!.actions() as Array<{
      choose?: { option?: { slot?: number; player?: string } };
    }>;
    const full = window.__arena!.full() as Full;
    return {
      hasChoose: legal.some((a) => "choose" in a),
      lastChoose: [...actions].reverse().find((a) => a.choose)?.choose?.option ?? null,
      def: full.players.b.field[1]?.defense ?? null,
    };
  });
  expect(after.hasChoose).toBeFalsy();
  expect(after.lastChoose).toEqual({ slot: 1, player: "b" });
  if (after.def != null && beforeDef != null) {
    expect(after.def).toBe(beforeDef - 5);
  }
});

test("allied follower choice highlights only the own board at colliding slots", async ({ page }) => {
  await boot(page);
  const selfDeck = await importDeck(page, "ally-self.json", { [QUICK]: 20, [REAPER]: 20 });
  const enemyDeck = await importDeck(page, "ally-enemy.json", { [QUICK]: 40 });
  await startGame(page, { first: "b", deckA: enemyDeck, deckB: selfDeck });
  await confirmMulligans(page);
  await closeDrawer(page);

  await driveToChoice(page, {
    caster: "b",
    spell: REAPER,
    ownNeed: 1,
    enemyNeed: 1,
    ownPlay: QUICK,
    enemyPlay: QUICK,
  });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  await assertHighlightsMatchNode(page, "b");
  await expect(page.locator("#blueBoard .card.legal-target, #blueBoard .card.selectable")).toHaveCount(0);
  await expect(page.locator("#redBoard .card.legal-target")).toHaveCount(1);
});
