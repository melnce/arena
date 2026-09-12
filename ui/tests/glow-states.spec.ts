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

async function attackLeader(page: Page) {
  const ok = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { player: string; target: unknown };
    }>;
    const act = legal.find((a) => a.attack?.player === "a" && a.attack.target === "leader");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(ok, "expected a leader attack").toBeTruthy();
}

async function wrapperGlow(page: Page, card: ReturnType<Page["locator"]>) {
  return card.locator(".card-image-wrapper").evaluate((el) => {
    const s = getComputedStyle(el);
    return {
      color: s.outlineColor,
      width: s.outlineWidth,
      animation: s.animationName,
    };
  });
}

test("glow matrix: plain / evolved / super-evolved × green / yellow / none", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  const me = await importDeck(page, "glow-matrix-a.json", { [FIGHTER]: 40 });
  const them = await importDeck(page, "glow-matrix-b.json", { [LEAH]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);

  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await playCard(page, LEAH);
  await applyFirst(page, "end_turn");
  await playCard(page, FIGHTER);
  const plain = page.locator("#blueBoard .card[data-card='10001110']").first();
  await expect(plain).toHaveClass(/rush-glow/);
  let g = await wrapperGlow(page, plain);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(plain, "yellow");

  await applyFirst(page, "end_turn");
  await expect(plain).not.toHaveClass(/rush-glow/);
  await expect(plain).not.toHaveClass(/can-attack/);
  await assertGlow(plain, "none");

  await applyFirst(page, "end_turn");
  await expect(plain).toHaveClass(/can-attack/);
  g = await wrapperGlow(page, plain);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(plain, "green");
  const plainPath = await artShot(plain, `${ART}/glow_matrix_plain_green.png`);
  assertPngLeftEdge(plainPath, "green");

  await attackLeader(page);
  await expect(plain).not.toHaveClass(/can-attack/);
  await assertGlow(plain, "none");

  for (let i = 0; i < 24; i++) {
    const snap = await page.evaluate(() => ({
      evo: window.__arena!.playerInfo("a").evolve_unlocked,
      active: (window.__arena!.full() as { active: string }).active,
    }));
    if (snap.active === "a" && snap.evo) break;
    await applyFirst(page, "end_turn");
  }
  await playCard(page, FIGHTER);
  const evoAct = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
    return legal.find((a) => a.evolve && !a.evolve.super) ?? null;
  });
  expect(evoAct).toBeTruthy();
  await page.evaluate((act) => window.__arena!.apply(act), evoAct);
  const evolved = page.locator("#blueBoard .card.evolved[data-card='10001110']").first();
  await expect(evolved).toHaveClass(/rush-glow/);
  g = await wrapperGlow(page, evolved);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(evolved, "yellow");

  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  await expect(evolved).toHaveClass(/can-attack/);
  g = await wrapperGlow(page, evolved);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(evolved, "green");
  const evoPath = await artShot(evolved, `${ART}/glow_matrix_evolved_green.png`);
  assertPngLeftEdge(evoPath, "green");

  await attackLeader(page);
  await assertGlow(evolved, "none");

  for (let i = 0; i < 24; i++) {
    const snap = await page.evaluate(() => ({
      se: window.__arena!.playerInfo("a").super_evolve_unlocked,
      active: (window.__arena!.full() as { active: string }).active,
      field: (window.__arena!.full() as { players: { a: { field: unknown[] } } }).players.a.field
        .filter(Boolean).length,
    }));
    if (snap.active === "a" && snap.se && snap.field === 0) break;
    if (snap.active === "a" && snap.field > 0) {
      const attacked = await page.evaluate(() => {
        const legal = window.__arena!.legal() as Array<{
          attack?: { player: string; target: unknown };
        }>;
        const act = legal.find((a) => a.attack?.player === "a" && a.attack.target === "leader");
        if (!act) return false;
        window.__arena!.apply(act);
        return true;
      });
      if (!attacked) await applyFirst(page, "end_turn");
      continue;
    }
    await applyFirst(page, "end_turn");
  }
  await playCard(page, FIGHTER);
  const seAct = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
    return legal.find((a) => a.evolve && a.evolve.super) ?? null;
  });
  expect(seAct, "super-evolve action").toBeTruthy();
  await page.evaluate((act) => window.__arena!.apply(act), seAct);
  const se = page.locator("#blueBoard .card.super-evo[data-card='10001110']").first();
  await expect(se).toHaveClass(/super-evo/);
  await expect(se).toHaveClass(/rush-glow/);
  g = await wrapperGlow(page, se);
  expect(g.color).toBe("rgb(255, 212, 0)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("enhpulse");
  await assertGlow(se, "yellow");
  await artShot(se, `${ART}/glow_matrix_super_yellow.png`);

  await applyFirst(page, "end_turn");
  await expect(se).not.toHaveClass(/can-attack/);
  await expect(se).not.toHaveClass(/rush-glow/);
  g = await wrapperGlow(page, se);
  expect(g.width === "0px" || g.color === "rgba(0, 0, 0, 0)").toBeTruthy();

  await applyFirst(page, "end_turn");
  await expect(se).toHaveClass(/can-attack/);
  g = await wrapperGlow(page, se);
  expect(g.color).toBe("rgb(57, 217, 138)");
  expect(g.width).toBe("4px");
  expect(g.animation.toLowerCase()).toContain("playpulse");
  await assertGlow(se, "green");
  const sePath = await artShot(se, `${ART}/glow_matrix_super_green.png`);
  assertPngLeftEdge(sePath, "green");

  await attackLeader(page);
  await assertGlow(se, "none");
});
