import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const MAJESTIC = "10622310";
const CRESCENT = "10441310";
const SANDALPHON = "10404110";
const ACCELERATE = "10844120";
const CRYSTALLIZE = "10661110";

const FOLLOWER_CROP = { fx: 0.478, fy: 0.338, w: 0.185 };
const SPELL_CROP = { fx: 0.5, fy: 0.4, w: 0.36 };
const APERTURE = { left: 0.1055, top: 0.1016, width: 0.7798, height: 0.7891 };
const CARD_ART = { width: 368, height: 473 };
const FRAME_ART = { width: 218, height: 256 };
const ICON_W = 56;

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

async function catalogCardUrl(page: Page, grantId: string): Promise<string> {
  const hash = await page.evaluate(async (id) => {
    const rec = (await fetch("catalog-images.json").then((r) => r.json())) as Record<
      string,
      { card?: string }
    >;
    return rec[id]?.card ?? "";
  }, grantId);
  expect(hash, `catalog hash for ${grantId}`).toBeTruthy();
  return `https://shadowverse-wb.com/uploads/card_image/eng/card/${hash}.png`;
}

async function assertIconGeometry(
  page: Page,
  cardId: string,
  crestId: string,
  grantId: string,
  crop: { fx: number; fy: number; w: number },
) {
  const tip = page.locator("#cardTooltip");
  await paintCardTooltip(page, cardId);
  const icon = tip.locator(".crest-icon");
  const portrait = icon.locator("img.crest-icon-portrait");
  const frame = icon.locator("img.crest-icon-frame");
  await expect(icon).toHaveCount(1);
  await expect(portrait).toHaveCount(1);
  await expect(frame).toHaveCount(1);
  await expect(portrait).toHaveAttribute("src", await catalogCardUrl(page, grantId));
  const geo = await icon.evaluate((el) => {
    const port = el.querySelector<HTMLImageElement>(".crest-icon-portrait");
    const frm = el.querySelector<HTMLImageElement>(".crest-icon-frame");
    if (!port || !frm) return null;
    const ps = getComputedStyle(port);
    const fs = getComputedStyle(frm);
    const box = el.getBoundingClientRect();
    const ap = el.querySelector(".crest-icon-aperture")?.getBoundingClientRect();
    return {
      iconW: box.width,
      iconH: box.height,
      apertureW: ap?.width ?? 0,
      apertureH: ap?.height ?? 0,
      width: Number.parseFloat(ps.width),
      height: Number.parseFloat(ps.height),
      left: Number.parseFloat(ps.left),
      top: Number.parseFloat(ps.top),
      fx: Number.parseFloat(getComputedStyle(el).getPropertyValue("--crest-fx")),
      fy: Number.parseFloat(getComputedStyle(el).getPropertyValue("--crest-fy")),
      zoom: Number.parseFloat(getComputedStyle(el).getPropertyValue("--crest-zoom")),
      frameZ: Number.parseFloat(fs.zIndex),
      portZ: Number.parseFloat(ps.zIndex),
    };
  });
  expect(geo).toBeTruthy();
  expect(geo!.iconW).toBeCloseTo(ICON_W, 0);
  expect(geo!.iconH).toBeCloseTo(geo!.iconW * (FRAME_ART.height / FRAME_ART.width), 0);
  expect(geo!.apertureW).toBeCloseTo(geo!.iconW * APERTURE.width, 0);
  expect(geo!.apertureH).toBeCloseTo(geo!.iconH * APERTURE.height, 0);
  expect(geo!.fx).toBeCloseTo(crop.fx, 5);
  expect(geo!.fy).toBeCloseTo(crop.fy, 5);
  expect(geo!.zoom).toBeCloseTo(crop.w, 5);
  const portraitW = geo!.iconW * APERTURE.width / crop.w;
  const portraitH = portraitW * (CARD_ART.height / CARD_ART.width);
  expect(geo!.width).toBeCloseTo(portraitW, 0);
  expect(geo!.height).toBeCloseTo(portraitH, 0);
  expect(geo!.left).toBeCloseTo(geo!.apertureW / 2 - geo!.width * crop.fx, 0);
  expect(geo!.top).toBeCloseTo(geo!.apertureH / 2 - geo!.height * crop.fy, 0);
  expect(geo!.frameZ).toBeGreaterThan(geo!.portZ);
  expect(crestId.startsWith("crest:") || crestId.startsWith("faith:")).toBeTruthy();
}

test("crest icon: granting-card crop geometry, frame on top, abort fallback", async ({ page }) => {
  await boot(page);
  const id = await importDeck(page, "icons.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  await assertIconGeometry(page, SANDALPHON, "crest:10404110", SANDALPHON, FOLLOWER_CROP);
  await assertIconGeometry(page, MAJESTIC, "crest:10622310", MAJESTIC, SPELL_CROP);
  await assertIconGeometry(page, CRESCENT, "crest:10441310", CRESCENT, SPELL_CROP);

  await page.route("https://shadowverse-wb.com/**", (route) => route.abort());
  await paintCardTooltip(page, MAJESTIC);
  const tip = page.locator("#cardTooltip");
  await expect(tip.locator(".crest-icon-frame")).toHaveCount(1);
  await expect
    .poll(async () =>
      tip.evaluate((el) => el.querySelectorAll("img.crest-icon-portrait").length),
    )
    .toBe(0);
  const broken = await tip.evaluate(
    (el) => [...el.querySelectorAll("img")].filter((img) => img.naturalWidth === 0).length,
  );
  expect(broken).toBe(0);
  await expect(tip.locator("div.tooltip-crest-icon")).toHaveCount(0);
});

test("crest icon contact sheet of all 42", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const id = await importDeck(page, "sheet.json", { [MAJESTIC]: 40 });
  await startGame(page, id);
  await confirmMulligans(page);
  await closeDrawer(page);

  const loaded = await page.evaluate(async () => {
    const spells = new Set([
      "crest:10412310",
      "crest:10441310",
      "crest:10451310",
      "crest:10453310",
      "crest:10553310",
      "crest:10622310",
      "crest:10712310",
      "crest:10713310",
    ]);
    const catalog = (await fetch("catalog-images.json").then((r) => r.json())) as Record<
      string,
      { card?: string }
    >;
    const ids = window.__arena!.catalogIds();
    const crests: Array<{ id: string; name: string; grantedBy?: string }> = [];
    const seen = new Set<string>();
    for (const cardId of ids) {
      for (const crest of window.__arena!.cardText(cardId).crests ?? []) {
        if (seen.has(crest.id)) continue;
        seen.add(crest.id);
        crests.push(crest);
      }
    }
    crests.sort((a, b) => a.id.localeCompare(b.id));

    const host = document.createElement("div");
    host.id = "crest-contact-sheet";
    host.style.cssText =
      "position:fixed;inset:0;z-index:20000;overflow:auto;background:#12141a;padding:16px;display:grid;grid-template-columns:repeat(7,minmax(0,1fr));gap:16px;";
    for (const crest of crests) {
      const fig = document.createElement("figure");
      fig.style.cssText = "margin:0;text-align:center;color:#e8edf5;font:12px/1.3 sans-serif;";
      const grant = crest.grantedBy || crest.id.replace(/^(?:crest|faith):/, "");
      const hash = catalog[grant]?.card;
      const crop = spells.has(crest.id)
        ? { fx: 0.5, fy: 0.4, w: 0.36 }
        : { fx: 0.478, fy: 0.338, w: 0.185 };
      const icon = document.createElement("span");
      icon.className = "crest-icon";
      icon.style.setProperty("--icon-w", "96px");
      icon.style.setProperty("--crest-fx", String(crop.fx));
      icon.style.setProperty("--crest-fy", String(crop.fy));
      icon.style.setProperty("--crest-zoom", String(crop.w));
      const aperture = document.createElement("span");
      aperture.className = "crest-icon-aperture";
      if (hash) {
        const img = document.createElement("img");
        img.className = "crest-icon-portrait crest-image";
        img.referrerPolicy = "no-referrer";
        img.src = `https://shadowverse-wb.com/uploads/card_image/eng/card/${hash}.png`;
        aperture.appendChild(img);
      }
      const frame = document.createElement("img");
      frame.className = "crest-icon-frame";
      frame.src = "crests/crest_frame.png";
      icon.append(aperture, frame);
      fig.appendChild(icon);
      const cap = document.createElement("figcaption");
      cap.textContent = crest.name;
      fig.appendChild(cap);
      host.appendChild(fig);
    }
    document.body.appendChild(host);
    const portraits = [...host.querySelectorAll<HTMLImageElement>(".crest-icon-portrait")];
    const results = await Promise.all(
      portraits.map(
        (img) =>
          new Promise<boolean>((resolve) => {
            if (img.complete) {
              resolve(img.naturalWidth > 0);
              return;
            }
            img.addEventListener("load", () => resolve(true), { once: true });
            img.addEventListener("error", () => resolve(false), { once: true });
            window.setTimeout(() => resolve(img.naturalWidth > 0), 6000);
          }),
      ),
    );
    const loaded = results.filter(Boolean).length;
    return { count: crests.length, art: loaded > 0, loaded };
  });

  expect(loaded.count).toBe(42);
  if (!loaded.art) {
    test.info().annotations.push({
      type: "note",
      description: "shadowverse-wb.com art did not load; contact sheet omitted",
    });
    return;
  }
  await artShot(page.locator("#crest-contact-sheet"), `${ART}/crest_icon_contact_sheet.png`, {
    fullPage: false,
  });
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
