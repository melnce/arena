import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, assertGlow, assertPngLeftEdge } from "./helpers.ts";

const FIGHTER = "10001110";
const LEAH = "10001120";

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

async function startGame(page: Page, deckA: string, deckB?: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckA);
  await page.locator("#redDeckSelect").selectOption(deckB ?? deckA);
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

async function attackOnce(page: Page) {
  const ok = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { player: string; target: unknown };
    }>;
    const act =
      legal.find((a) => a.attack?.player === "a" && a.attack.target === "leader") ??
      legal.find((a) => a.attack?.player === "a");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(ok, "expected an attack").toBeTruthy();
}

async function wrapperGlow(card: ReturnType<Page["locator"]>) {
  return card.locator(".card-image-wrapper").evaluate((el) => {
    const s = getComputedStyle(el);
    return {
      color: s.outlineColor,
      width: s.outlineWidth,
      animation: s.animationName,
    };
  });
}

async function readyFighter(page: Page, deckA: string, deckB: string) {
  await startGame(page, deckA, deckB);
  await confirmMulligans(page);
  await closeDrawer(page);
  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await playCard(page, FIGHTER);
  await applyFirst(page, "end_turn");
  await playCard(page, FIGHTER);
  await applyFirst(page, "end_turn");
  return page.locator("#blueBoard .card[data-card='10001110']").first();
}

test("plain follower: green when it can hit the leader, yellow with a printed lock, none after attacking", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await boot(page);
  const me = await importDeck(page, "glow-plain-a.json", { [FIGHTER]: 40 });
  const them = await importDeck(page, "glow-plain-b.json", { [FIGHTER]: 40 });
  const card = await readyFighter(page, me, them);
  await expect(card).toHaveClass(/can-attack/);
  let g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(card, "green");
  const greenPath = await artShot(card, `${ART}/glow_matrix_plain_green.png`);
  assertPngLeftEdge(greenPath, "green");

  const slot = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ slot: number }>;
    return info[0]?.slot ?? 0;
  });
  await page.evaluate((s) => window.__arena!.debugGrantCantAttackLeader("a", s), slot);
  await expect(card).toHaveClass(/rush-glow/);
  g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(card, "yellow");
  await artShot(card, `${ART}/glow_matrix_plain_yellow.png`);

  await applyFirst(page, "end_turn");
  await expect(card).not.toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "none");
});

test("evolved follower keeps the attack ring over the E badge", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const me = await importDeck(page, "glow-evo-a.json", { [FIGHTER]: 40 });
  const them = await importDeck(page, "glow-evo-b.json", { [LEAH]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

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
        canPlayA: legal.some((a) => a.play?.card === cardId),
        canPlayB: legal.some((a) => a.play?.card === "10001120"),
        canEnd: legal.some((a) => a.end_turn),
      };
    }, FIGHTER);
    if (snap.active === "a" && snap.evo && snap.enemy >= 1 && snap.own === 0 && snap.canPlayA) {
      await playCard(page, FIGHTER);
      break;
    }
    if (snap.active === "b" && snap.enemy < 1 && snap.canPlayB) {
      await playCard(page, LEAH);
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

  const card = page.locator("#blueBoard .card.evolved[data-card='10001110']").first();
  await expect(card).toHaveClass(/rush-glow/);
  let g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(card, "yellow");

  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/can-attack/);
  g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(card, "green");
  const path = await artShot(card, `${ART}/glow_matrix_evolved_green.png`);
  assertPngLeftEdge(path, "green");

  await attackOnce(page);
  await assertGlow(card, "none");
});

test("super-evolved follower keeps the attack ring over the purple SE chrome", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  const me = await importDeck(page, "glow-se-a.json", { [FIGHTER]: 40 });
  const them = await importDeck(page, "glow-se-b.json", { [LEAH]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

  for (let i = 0; i < 40; i++) {
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
        se: window.__arena!.playerInfo("a").super_evolve_unlocked,
        enemy: full.players.b.field.filter(Boolean).length,
        own: full.players.a.field.filter(Boolean).length,
        canPlayA: legal.some((a) => a.play?.card === cardId),
        canPlayB: legal.some((a) => a.play?.card === "10001120"),
        canEnd: legal.some((a) => a.end_turn),
      };
    }, FIGHTER);
    if (snap.active === "a" && snap.se && snap.enemy >= 1 && snap.own === 0 && snap.canPlayA) {
      await playCard(page, FIGHTER);
      break;
    }
    if (snap.active === "b" && snap.enemy < 1 && snap.canPlayB) {
      await playCard(page, LEAH);
      continue;
    }
    if (snap.canEnd) {
      await applyFirst(page, "end_turn");
      continue;
    }
    throw new Error(`stuck driving to super-evolve-on-play (i=${i})`);
  }

  const seAct = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
    return legal.find((a) => a.evolve && a.evolve.super) ?? null;
  });
  expect(seAct, "super-evolve action").toBeTruthy();
  await page.evaluate((act) => window.__arena!.apply(act), seAct);

  const card = page.locator("#blueBoard .card.super-evo[data-card='10001110']").first();
  await expect(card).toHaveClass(/super-evo/);
  await expect(card).toHaveClass(/rush-glow/);
  let g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(card, "yellow");
  await artShot(card, `${ART}/glow_matrix_super_yellow.png`);

  await applyFirst(page, "end_turn");
  await expect(card).not.toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "none");

  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/can-attack/);
  g = await wrapperGlow(card);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(card, "green");
  const path = await artShot(card, `${ART}/glow_matrix_super_green.png`);
  assertPngLeftEdge(path, "green");

  await attackOnce(page);
  await assertGlow(card, "none");
});
