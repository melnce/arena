import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, YELLOW } from "./helpers.ts";

const LEONA = "10871120";
const HAIKUMASTER = "10532120";
const REAPERS_DUE = "10953310";
const FANFARE_LAST_WORDS = "10641110";
const FILLER = "10622310";

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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function paintCardTooltip(page: Page, cardId: string): Promise<void> {
  await page.evaluate((id) => {
    let el = document.getElementById("crest-text-probe");
    if (!el) {
      el = document.createElement("div");
      el.id = "crest-text-probe";
      el.className = "card";
      el.style.cssText = "position:fixed;left:8px;top:8px;width:40px;height:40px;z-index:1;";
      document.body.appendChild(el);
    }
    el.dataset.card = id;
    el.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
  }, cardId);
  await expect(page.locator("#cardTooltip")).toBeVisible();
}

async function descLines(page: Page): Promise<string[]> {
  return page.locator("#cardTooltip .tooltip-desc-line").allTextContents();
}

test("Leona Super-Evolve is one yellow keyword line", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "leona.json", { [LEONA]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await paintCardTooltip(page, LEONA);
  const tip = page.locator("#cardTooltip");
  const lines = tip.locator(".tooltip-desc-line");
  await expect(lines).toHaveCount(2);
  await expect(lines.nth(1)).toContainText(/^Super-Evolve:/);
  const texts = await lines.allTextContents();
  expect(texts.some((t) => t.trim() === "Super-")).toBeFalsy();

  const kw = lines.nth(1).locator(".tooltip-keyword").filter({ hasText: /^Super-Evolve/ });
  await expect(kw).toHaveCount(1);
  const color = await kw.evaluate((el) => getComputedStyle(el).color);
  expect(color).toBe(YELLOW);

  await artShot(tip, `${ART}/leona_superevolve_tooltip.png`);
});

test("sweep: every Super-Evolve catalog card stays one keyword line", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const id = await importDeck(page, "se-sweep.json", { [FILLER]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const cards = await page.evaluate(() => {
    const ids = window.__arena!.catalogIds();
    return ids
      .map((cardId) => {
        const info = window.__arena!.cardText(cardId);
        return { id: cardId, text: info.text ?? "" };
      })
      .filter((row) => /super[- ]evolve:/i.test(row.text));
  });

  expect(cards.length, "cards whose text matches /super[- ]evolve:/i").toBe(60);

  const tip = page.locator("#cardTooltip");
  for (const row of cards) {
    await paintCardTooltip(page, row.id);
    const lines = tip.locator(".tooltip-desc-line");
    const texts = (await lines.allTextContents()).map((t) => t.trim());
    expect(texts, row.id).not.toContain("Super-");
    expect(texts, row.id).not.toContain("Super");
    const se = lines.filter({ hasText: /^Super-Evolve/ });
    await expect(se, row.id).toHaveCount(1);
    await expect(
      se.locator(".tooltip-keyword").filter({ hasText: /^Super-Evolve/ }),
      row.id,
    ).toHaveCount(1);
  }
});

test("On Spellboost stays one line", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "haiku.json", { [HAIKUMASTER]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await paintCardTooltip(page, HAIKUMASTER);
  const lines = await descLines(page);
  expect(lines.some((t) => t.trim() === "On")).toBeFalsy();
  expect(lines.some((t) => t.trim().startsWith("On Spellboost:"))).toBeTruthy();
});

test("Reaper's Due quoted Last Words stays on one line", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "reaper.json", { [REAPERS_DUE]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await paintCardTooltip(page, REAPERS_DUE);
  const lines = await descLines(page);
  const joined = lines.join("\n");
  expect(joined).toMatch(/Last Words:\s*Summon a copy of this card/);
  expect(lines.some((t) => /^Last Words:/.test(t.trim()))).toBeFalsy();
});

test("Fanfare + Last Words on separate lines stay two lines", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "two-abilities.json", { [FANFARE_LAST_WORDS]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await paintCardTooltip(page, FANFARE_LAST_WORDS);
  const lines = await descLines(page);
  expect(lines.length).toBeGreaterThanOrEqual(2);
  expect(lines.some((t) => t.trim().startsWith("Fanfare:"))).toBeTruthy();
  expect(lines.some((t) => t.trim().startsWith("Last Words:"))).toBeTruthy();
});
