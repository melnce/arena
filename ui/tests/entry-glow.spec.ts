import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, assertGlow, assertPngLeftEdge } from "./helpers.ts";

const FIGHTER = "10001110";

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

async function startGame(page: Page, deckId: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
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
  await applyFirst(page, "end_turn");
  await expect(card).toHaveClass(/can-attack/);
  await expect(card).not.toHaveClass(/rush-glow/);
  await assertGlow(card, "green");
  const greenPath = await artShot(card, `${ART}/entry_evo_green.png`);
  assertPngLeftEdge(greenPath, "green");
});
