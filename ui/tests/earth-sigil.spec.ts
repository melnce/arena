import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const BREW = "10031210";
const WIZARD = "10531110";
const SNACK = "10732310";

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

async function chooseEarthRite2(page: Page) {
  const viaLegal = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      choose?: { option?: { mode?: number } | number };
    }>;
    const act = legal.find((a) => {
      const o = a.choose?.option;
      return typeof o === "object" && o != null && o.mode === 1;
    });
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  if (viaLegal) return;
  const btn = page.locator(".choice-option").filter({ hasText: /Earth Rite/i });
  if (await btn.count()) {
    await btn.first().click();
    return;
  }
  if (await page.locator(".choice-option").count()) {
    await page.locator(".choice-option").nth(1).click();
  }
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

async function skipToPp(page: Page, pp: number) {
  for (let i = 0; i < 24; i++) {
    const ready = await page.evaluate((need) => {
      const full = window.__arena!.full() as {
        players: { a: { pp: number } };
        active: string;
      };
      return full.active === "a" && full.players.a.pp >= need;
    }, pp);
    if (ready) return;
    await applyFirst(page, "end_turn");
  }
}

async function earthState(page: Page) {
  return page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { earth: number; earth_slot: number | null; field: Array<{ card: string } | null> } };
    };
    const p = full.players.a;
    const slot = p.earth_slot;
    const card = slot != null ? p.field[slot]?.card ?? null : null;
    return { earth: p.earth, slot, card };
  });
}

async function expectBadge(page: Page, card: string, n: number) {
  const st = await earthState(page);
  expect(st.earth).toBe(n);
  expect(st.card).toBe(card);
  const badge = page.locator(`#blueBoard .card[data-card='${card}'] .earth-sigil-badge`);
  await expect(badge).toHaveText(String(n));
}

test("Earth Sigil stack badge on Witch's New Brew follows full().earth", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "earth-mix.json", {
    [BREW]: 16,
    [WIZARD]: 12,
    [SNACK]: 12,
  });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await playCard(page, BREW);
  await expectBadge(page, BREW, 1);
  const brew = page.locator("#blueBoard .card[data-card='10031210']").first();
  await brew.hover();
  await expect(page.locator("#cardTooltip .tooltip-earth-sigils")).toHaveText("Earth Sigils: 1");
  await artShot(brew, `${ART}/earth_sigil_brew_1.png`);

  await skipToPp(page, 1);
  await expect(brew).toHaveClass(/engage-ready/);
  const engaged = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ engage?: { player: string } }>;
    const act = legal.find((a) => a.engage?.player === "a");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(engaged, "engage brew").toBeTruthy();
  await expectBadge(page, BREW, 2);
  await artShot(brew, `${ART}/earth_sigil_brew_2.png`);

  await skipToPp(page, 3);
  await playCard(page, WIZARD);
  await expectBadge(page, BREW, 4);
  await artShot(brew, `${ART}/earth_sigil_brew_4.png`);

  await skipToPp(page, 2);
  await playCard(page, SNACK);
  await chooseEarthRite2(page);
  await expect.poll(async () => (await earthState(page)).earth).toBe(2);
  await expectBadge(page, BREW, 2);
  await artShot(brew, `${ART}/earth_sigil_brew_rite2.png`);
});

test("Magic Sediment holds the stack when no Brew is on the field", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "earth-sediment.json", { [WIZARD]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);
  await skipToPp(page, 3);
  await playCard(page, WIZARD);
  const st = await earthState(page);
  expect(st.card).toBe("90031210");
  expect(st.earth).toBeGreaterThan(0);
  const badge = page.locator("#blueBoard .card[data-card='90031210'] .earth-sigil-badge");
  await expect(badge).toHaveText(String(st.earth));
  await artShot(page.locator("#blueBoard .card[data-card='90031210']"), `${ART}/earth_sigil_sediment.png`);
});

test("Earth Sigil amulet leaves when the stack reaches 0", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "earth-zero.json", { [BREW]: 20, [SNACK]: 20 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, BREW);
  await skipToPp(page, 1);
  const engaged = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ engage?: { player: string } }>;
    const act = legal.find((a) => a.engage?.player === "a");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(engaged).toBeTruthy();
  await expectBadge(page, BREW, 2);
  await skipToPp(page, 2);
  await playCard(page, SNACK);
  await chooseEarthRite2(page);
  await expect.poll(async () => (await earthState(page)).earth).toBe(0);
  await expect(page.locator("#blueBoard .card[data-card='10031210']")).toHaveCount(0);
});
