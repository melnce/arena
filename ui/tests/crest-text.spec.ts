import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const MAJESTIC = "10622310";
const CRESCENT = "10441310";
const ACCELERATE = "10844120";
const CRYSTALLIZE = "10661110";

function normWs(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

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

test("sweep: every crest-granting catalog card paints name + text panels", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const id = await importDeck(page, "crest-sweep.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const cards = await page.evaluate(() => {
    const ids = window.__arena!.catalogIds();
    return ids
      .map((cardId) => {
        const info = window.__arena!.cardText(cardId);
        return { id: cardId, crests: info.crests ?? [] };
      })
      .filter((row) => row.crests.length > 0);
  });

  expect(cards.length, "cards with ≥1 crest").toBe(42);
  const crestIds = new Set(cards.flatMap((row) => row.crests.map((c) => c.id)));
  expect(crestIds.size, "distinct crests across those cards").toBe(42);

  const tip = page.locator("#cardTooltip");
  for (const row of cards) {
    await paintCardTooltip(page, row.id);
    const panels = tip.locator(".tooltip-crest-panel");
    await expect(panels, row.id).toHaveCount(row.crests.length);
    for (let i = 0; i < row.crests.length; i++) {
      const panel = panels.nth(i);
      const name = panel.locator(".tooltip-crest-name");
      const body = panel.locator(".tooltip-crest-text");
      await expect(name, `${row.id} name`).toHaveText(row.crests[i].name);
      const got = normWs(await body.innerText());
      expect(got, `${row.id} text`).toBe(normWs(row.crests[i].text));
      expect(got, `${row.id} empty body`).not.toBe("");
      expect(await name.innerText(), `${row.id} numeric title`).not.toMatch(/^\d+$/);
    }
  }
});

test("Majestic Conquest hand tooltip: one crest panel with real text", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "majestic.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const card = page.locator(`.card[data-card="${MAJESTIC}"]`).first();
  await expect(card).toBeVisible();
  await card.hover();
  const tip = page.locator("#cardTooltip");
  await expect(tip).toBeVisible();
  await expect(tip.locator(".tooltip-crest-panel")).toHaveCount(1);
  await expect(tip.locator(".tooltip-crest-name")).toHaveText("Crest: Majestic Conquest");
  const body = normWs(await tip.locator(".tooltip-crest-text").innerText());
  expect(body).toContain("Countdown (2)");
  expect(body).toContain("Whenever you play an Enhanced card, summon a Fearless Soldier.");
  await artShot(tip, `${ART}/majestic_conquest_crest_tooltip.png`);
});

test("Accelerate / Crystallize form lines; Enhance stays once", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "forms.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const tip = page.locator("#cardTooltip");
  await paintCardTooltip(page, ACCELERATE);
  await expect(tip).toContainText("Accelerate (3): Gain 1 max play point.");
  await artShot(tip, `${ART}/accelerate_form_line.png`);

  await paintCardTooltip(page, CRYSTALLIZE);
  await expect(tip).toContainText("Crystallize (2):");
  await expect(tip).toContainText("Last Words: Summon a Prostrating Coward.");

  await paintCardTooltip(page, MAJESTIC);
  const enhanceHits = await tip.evaluate((el) => {
    const text = el.textContent ?? "";
    return text.split("Enhance (3):").length - 1;
  });
  expect(enhanceHits).toBe(1);
});

test("crest icon: art loads; missing art uses frame, never an empty box", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "icons.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const tip = page.locator("#cardTooltip");
  await paintCardTooltip(page, CRESCENT);
  const art = tip.locator("img.tooltip-crest-icon");
  await expect(art).toHaveCount(1);
  await expect
    .poll(async () => art.evaluate((el) => (el as HTMLImageElement).naturalWidth))
    .toBeGreaterThan(0);
  await expect(art).toHaveAttribute("src", /crescent_tube_ride\.png/);

  await paintCardTooltip(page, MAJESTIC);
  const frame = tip.locator("img.tooltip-crest-icon");
  await expect(frame).toHaveCount(1);
  await expect(frame).toHaveAttribute("src", /crest_frame\.png/);
  await expect
    .poll(async () => frame.evaluate((el) => (el as HTMLImageElement).naturalWidth))
    .toBeGreaterThan(0);
  const broken = await tip.evaluate((el) =>
    [...el.querySelectorAll("img")].filter((img) => img.naturalWidth === 0).length,
  );
  expect(broken).toBe(0);
  const emptyBox = tip.locator("div.tooltip-crest-icon");
  await expect(emptyBox).toHaveCount(0);
  await artShot(tip, `${ART}/majestic_crest_frame_fallback.png`);
});

test.describe("crest slot long-press", () => {
  test.use({ hasTouch: true });

  test("long-press opens crest tooltip; drag cancels; tap outside dismisses; mouse still works", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await boot(page);
    const id = await importDeck(page, "crest-touch.json", { [MAJESTIC]: 40 });
    await startGame(page, id);
    await confirmMulligans(page);
    await closeDrawer(page);

    const played = await page.evaluate((card) => {
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const act = legal.find((a) => a.play?.card === card);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    }, MAJESTIC);
    expect(played, "play Majestic Conquest").toBeTruthy();

    const slot = page.locator("#blueCrests .crest-slot[data-crest]").first();
    await expect(slot).toBeVisible();
    const tip = page.locator("#cardTooltip");

    const box = await slot.boundingBox();
    expect(box).toBeTruthy();
    const x = box!.x + box!.width / 2;
    const y = box!.y + box!.height / 2;

    await page.evaluate(({ px, py }) => {
      const el = document.elementFromPoint(px, py);
      el?.dispatchEvent(
        new PointerEvent("pointerdown", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: px,
          clientY: py,
        }),
      );
    }, { px: x, py: y });
    await page.waitForTimeout(450);
    await expect(tip).toBeVisible();
    await expect(tip.locator(".tooltip-header-name")).toHaveText("Crest: Majestic Conquest");
    await expect(tip).toContainText("Countdown (2)");
    await expect(tip).toContainText("Whenever you play an Enhanced card, summon a Fearless Soldier.");

    await page.evaluate(({ px, py }) => {
      document.dispatchEvent(
        new PointerEvent("pointerup", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: px,
          clientY: py,
        }),
      );
    }, { px: x, py: y });

    await page.evaluate(() => {
      const tipEl = document.getElementById("cardTooltip");
      if (tipEl) tipEl.style.display = "none";
    });

    await page.evaluate(({ px, py }) => {
      const el = document.elementFromPoint(px, py);
      el?.dispatchEvent(
        new PointerEvent("pointerdown", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: px,
          clientY: py,
        }),
      );
      document.dispatchEvent(
        new PointerEvent("pointermove", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: px + 20,
          clientY: py,
        }),
      );
    }, { px: x, py: y });
    await page.waitForTimeout(450);
    await expect(tip).toBeHidden();

    await page.evaluate(({ px, py }) => {
      const el = document.elementFromPoint(px, py);
      el?.dispatchEvent(
        new PointerEvent("pointerdown", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: px,
          clientY: py,
        }),
      );
    }, { px: x, py: y });
    await page.waitForTimeout(450);
    await expect(tip).toBeVisible();
    await page.evaluate(() => {
      document.body.dispatchEvent(
        new PointerEvent("pointerup", {
          bubbles: true,
          cancelable: true,
          pointerType: "touch",
          clientX: 8,
          clientY: 8,
        }),
      );
    });
    await expect(tip).toBeHidden();

    await slot.hover();
    await expect(tip).toBeVisible();
    await expect(tip.locator(".tooltip-header-name")).toHaveText("Crest: Majestic Conquest");
    await expect(tip).toContainText("Countdown (2)");
  });
});
