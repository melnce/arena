import { expect, test, type Page } from "@playwright/test";
import { mkdir } from "node:fs/promises";
import { ART, artShot, assertGlow, waitCardSizeStable } from "./helpers.ts";

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
  opts: {
    mode?: string;
    seed?: string;
    first?: string;
    deckA?: string;
    deckB?: string;
  } = {},
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode ?? "hotseat");
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
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

async function endVisibleTurn(page: Page) {
  const before = await page.locator("#turnCounter").getAttribute("data-acting");
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
    document.querySelector(".choice-modal")?.remove();
    const acting = document.getElementById("turnCounter")?.dataset.acting;
    const id = acting === "a" ? "endTurnBlue" : "endTurnRed";
    const btn = document.getElementById(id) as HTMLButtonElement | null;
    btn?.click();
  });
  await expect(page.locator("#turnCounter")).not.toHaveAttribute("data-acting", before ?? "", {
    timeout: 8000,
  });
}

test("bonus PP: first A, button on red, toggle then End Turn commits", async ({ page }) => {
  await boot(page);
  await startGame(page, { seed: "1", first: "a", deckA: "basic-forest", deckB: "basic-rune" });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
  await expect(page.locator("#redBoostHost #bonusPpBtn")).toHaveCount(1);
  await expect(page.locator("#blueBoostHost #bonusPpBtn")).toHaveCount(0);
  await expect(page.locator("#bonusPpBtn")).toBeDisabled();

  await page.locator("#endTurnBlue:visible").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 5000 });
  const btn = page.locator("#redBoostHost #bonusPpBtn");
  await expect(btn).toBeEnabled();
  const before = await page.locator("#redPP").innerText();
  expect(before).toMatch(/^\d+\/\d+$/);
  const [pp0, max0] = before.split("/").map(Number);

  await btn.click();
  await expect(page.locator("#redPP")).toHaveText(`${pp0 + 1}/${max0}`, { timeout: 5000 });
  await expect(btn).toHaveClass(/used/);

  await btn.click();
  await expect(page.locator("#redPP")).toHaveText(`${pp0}/${max0}`, { timeout: 5000 });

  await btn.click();
  await expect(page.locator("#redPP")).toHaveText(`${pp0 + 1}/${max0}`, { timeout: 5000 });
  await page.locator("#endTurnRed:visible").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 5000 });
  await expect(page.locator("#blueBoostHost #bonusPpBtn")).toHaveCount(0);
  await expect(page.locator("#redBoostHost #bonusPpBtn")).toHaveCount(1);
  await expect(page.locator("#bonusPpBtn")).toBeDisabled();
});

test("bonus PP: first B, button on blue never on first player", async ({ page }) => {
  await boot(page);
  await startGame(page, { seed: "1", first: "b", deckA: "basic-forest", deckB: "basic-rune" });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b");
  await expect(page.locator("#blueBoostHost #bonusPpBtn")).toHaveCount(1);
  await expect(page.locator("#redBoostHost #bonusPpBtn")).toHaveCount(0);
  await expect(page.locator("#bonusPpBtn")).toBeDisabled();

  await page.locator("#endTurnRed:visible").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 5000 });
  const btn = page.locator("#blueBoostHost #bonusPpBtn");
  await expect(btn).toBeEnabled();
  const before = await page.locator("#bluePP").innerText();
  const [pp0, max0] = before.split("/").map(Number);
  await btn.click();
  await expect(page.locator("#bluePP")).toHaveText(`${pp0 + 1}/${max0}`, { timeout: 5000 });
  await mkdir(ART, { recursive: true });
  await artShot(page.locator("#turnControls"), `${ART}/boost_on_blue_second.png`);
});

test("tooltips and gates: Hark necromancy + Depths enhance (no E badge)", async ({ page }) => {
  await boot(page);
  const harkId = await importDeck(page, "hark.json", { "10753310": 40 });
  await page.locator("#redDeckSelect").selectOption(harkId);
  await startGame(page, { seed: "1", first: "a", deckA: harkId, deckB: harkId });
  await confirmMulligans(page);

  const harkInfo = await page.evaluate(() => {
    const rows = window.__arena!.handInfo("a") as Array<{
      id: string;
      gates: Array<{ kind: string; need: number; have: number; met: boolean }>;
    }>;
    return rows.find((r) => r.id === "10753310") ?? rows[0];
  });
  expect(harkInfo.id).toBe("10753310");
  const necro = harkInfo.gates.find((g) => g.kind === "necromancy");
  expect(necro).toBeTruthy();
  expect(necro!.need).toBe(6);
  expect(necro!.have).toBe(0);
  expect(necro!.met).toBe(false);

  const harkCard = page.locator("#blueHand .card[data-card='10753310']").first();
  await harkCard.hover();
  const tip = page.locator("#cardTooltip");
  await expect(tip).toBeVisible();
  await expect(tip).toContainText("Hark to the Night Song");
  await expect(tip).toContainText("Necromancy 0/6");
  await expect(tip.locator(".tooltip-keyword").first()).toBeVisible();

  const depthsId = await importDeck(page, "depths.json", { "90024320": 40 });
  await page.locator("#redDeckSelect").selectOption(depthsId);
  await startGame(page, { seed: "1", first: "a", deckA: depthsId, deckB: depthsId });
  await confirmMulligans(page);

  const depthsInfo = await page.evaluate(() => {
    const rows = window.__arena!.handInfo("a") as Array<{
      id: string;
      cost: number | null;
      form: string | null;
      gates: Array<{ kind: string; need: number; have: number; met: boolean }>;
    }>;
    return rows.find((r) => r.id === "90024320") ?? rows[0];
  });
  expect(depthsInfo.id).toBe("90024320");
  expect(depthsInfo.form).toBe("enhance");
  expect(depthsInfo.cost).toBe(1);
  expect(depthsInfo.gates.find((g) => g.kind === "enhance")).toBeFalsy();

  const depthsCard = page.locator("#blueHand .card[data-card='90024320']").first();
  await expect(depthsCard.locator(".alternate-form-badge")).toHaveCount(0);
  await expect(depthsCard.locator(".cost-badge, .card-stats.top-left")).toHaveText("1");
  const depthsLegal = await page.evaluate(() =>
    (window.__arena!.legal() as Array<{ play?: { card: string } }>).some(
      (a) => a.play?.card === "90024320",
    ),
  );
  if (depthsLegal) {
    await expect(depthsCard).toHaveClass(/enhance-ready/);
    await assertGlow(depthsCard, "yellow");
  } else {
    await expect(depthsCard).not.toHaveClass(/playable-glow|enhance-ready|legal-play/);
    await assertGlow(depthsCard, "none");
  }
  await depthsCard.hover();
  await expect(tip).toContainText("Depths of the Eld Sword");
  await expect(tip).toContainText("Cost 1");
  await expect(tip).toContainText("base 0");
  await expect(tip).not.toContainText("Enhance 1 (have");
  await expect(tip).not.toContainText("0/0");
  await mkdir(ART, { recursive: true });
  await artShot(tip, `${ART}/tooltip_depths_gates.png`);
  await artShot(depthsCard, `${ART}/card_enhance_no_badge.png`);
});

test("settings drawer does not scroll horizontally at 360 and 768", async ({ page }) => {
  await boot(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await openSettings(page);
  const drawer = page.locator("#settingsDrawer");
  await expect(drawer).toHaveCSS("width", "360px");
  const overflow360 = await drawer.evaluate((el) => {
    const style = getComputedStyle(el);
    const right = el.getBoundingClientRect().right;
    let farthest = 0;
    for (const node of el.querySelectorAll<HTMLElement>("button, select, input, label, span")) {
      farthest = Math.max(farthest, node.getBoundingClientRect().right);
    }
    return {
      client: el.clientWidth,
      scroll: el.scrollWidth,
      overflowX: style.overflowX,
      spill: farthest - right,
    };
  });
  expect(overflow360.overflowX).toBe("hidden");
  expect(overflow360.spill).toBeLessThanOrEqual(2);
  await mkdir(ART, { recursive: true });
  await artShot(drawer, `${ART}/drawer_360.png`);

  await page.setViewportSize({ width: 768, height: 1024 });
  await openSettings(page);
  const overflow768 = await drawer.evaluate((el) => {
    const right = el.getBoundingClientRect().right;
    let farthest = 0;
    for (const node of el.querySelectorAll<HTMLElement>("button, select, input, label, span")) {
      farthest = Math.max(farthest, node.getBoundingClientRect().right);
    }
    return {
      width: el.getBoundingClientRect().width,
      overflowX: getComputedStyle(el).overflowX,
      spill: farthest - right,
    };
  });
  expect(overflow768.width).toBeGreaterThanOrEqual(767);
  expect(overflow768.overflowX).toBe("hidden");
  expect(overflow768.spill).toBeLessThanOrEqual(2);
  await artShot(drawer, `${ART}/drawer_768.png`);
});

test("board occupied slots are centred (1 / 2 / 3 followers)", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const id = await importDeck(page, "crystalspawn.json", { "10631110": 40 });
  await page.locator("#redDeckSelect").selectOption(id);
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);

  async function playBlueOne() {
    const card = page.locator("#blueHand .card.legal-play").first();
    await expect(card).toBeVisible({ timeout: 5000 });
    const before = await page.locator("#blueBoard .card").count();
    await card.click();
    const choice = page.locator(".choice-modal .choice-option, .confirm-targets-btn");
    if (await choice.count()) await choice.first().click();
    await expect(page.locator("#blueBoard .card")).toHaveCount(before + 1, { timeout: 8000 });
  }

  async function shot(n: number) {
    const board = page.locator("#blueBoard");
    await expect(board.locator(".card")).toHaveCount(n, { timeout: 5000 });
    await expect(board.locator(".empty-slot, .board-slot:not(:has(.card))")).toHaveCount(0);
    const geom = await board.evaluate((el) => {
      const cards = [...el.querySelectorAll<HTMLElement>(".card")];
      const br = el.getBoundingClientRect();
      const left = Math.min(...cards.map((c) => c.getBoundingClientRect().left));
      const right = Math.max(...cards.map((c) => c.getBoundingClientRect().right));
      return {
        mid: (left + right) / 2,
        boardMid: (br.left + br.right) / 2,
        count: cards.length,
      };
    });
    expect(geom.count).toBe(n);
    expect(Math.abs(geom.mid - geom.boardMid)).toBeLessThan(24);
    await page.evaluate(() => {
      const tip = document.getElementById("cardTooltip");
      if (tip) tip.style.display = "none";
    });
    await artShot(page.locator("#appRoot"), `${ART}/board_${n}_followers.png`);
  }

  await playBlueOne();
  await shot(1);
  await endVisibleTurn(page);
  await endVisibleTurn(page);
  await playBlueOne();
  await shot(2);
  await endVisibleTurn(page);
  await endVisibleTurn(page);
  await playBlueOne();
  await shot(3);
});

test("layouts fill the viewport at 720p / 1080p / 1440p", async ({ page }) => {
  await boot(page);
  await startGame(page, { seed: "1", first: "a", deckA: "basic-forest", deckB: "basic-rune" });
  await confirmMulligans(page);
  await mkdir(ART, { recursive: true });

  const viewports = [
    { name: "720p", width: 1280, height: 720 },
    { name: "1080p", width: 1920, height: 1080 },
    { name: "1440p", width: 2560, height: 1440 },
  ] as const;

  for (const vp of viewports) {
    await page.setViewportSize({ width: vp.width, height: vp.height });
    await page.evaluate(() => {
      const tip = document.getElementById("cardTooltip");
      if (tip) tip.style.display = "none";
    });
    await waitCardSizeStable(page);
    const metrics = await page.evaluate(() => {
      const ids = ["redHand", "redLeader", "redBoard", "blueBoard", "blueLeader", "blueHand"];
      const rects = ids.map((id) => {
        const el = document.getElementById(id);
        return el ? el.getBoundingClientRect() : null;
      });
      const present = rects.filter((r): r is DOMRect => !!r);
      const top = Math.min(...present.map((r) => r.top));
      const bottom = Math.max(...present.map((r) => r.bottom));
      const fill = (bottom - top) / window.innerHeight;
      const rowOverlap = present
        .slice()
        .sort((a, b) => a.top - b.top)
        .some((r, i, arr) => i > 0 && r.top + 2 < arr[i - 1].bottom);
      const redBoard = rects[2];
      const blueBoard = rects[3];
      const boardGap =
        redBoard && blueBoard ? Math.max(0, blueBoard.top - redBoard.bottom) : 999;
      const red = document.querySelector<HTMLElement>("#redHand .card");
      const blue = document.querySelector<HTMLElement>("#blueHand .card");
      const cardH = blue?.getBoundingClientRect().height ?? 0;
      const hand = document.getElementById("blueHand");
      const cards = hand ? [...hand.querySelectorAll<HTMLElement>(".card")] : [];
      let handOverlapFrac = 0;
      if (cards.length >= 2) {
        const a = cards[0].getBoundingClientRect();
        const b = cards[1].getBoundingClientRect();
        const gap = b.left - a.right;
        handOverlapFrac = gap >= 0 ? 0 : Math.abs(gap) / a.width;
      }
      return {
        redH: red?.getBoundingClientRect().height ?? 0,
        blueH: cardH,
        fill,
        rowOverlap,
        boardGap,
        cardH,
        handOverlapFrac,
      };
    });
    expect(metrics.redH).toBeGreaterThan(0);
    expect(metrics.blueH).toBeGreaterThan(0);
    expect(Math.abs(metrics.redH - metrics.blueH)).toBeLessThan(2);
    expect(metrics.fill).toBeGreaterThanOrEqual(0.85);
    expect(metrics.rowOverlap).toBe(false);
    expect(metrics.boardGap).toBeLessThanOrEqual(metrics.cardH + 1);
    expect(metrics.handOverlapFrac).toBeLessThanOrEqual(0.36);
    await artShot(page, `${ART}/layout_${vp.name}.png`);
  }
});

test("boost button side screenshot + paintMs for a turn", async ({ page }) => {
  await boot(page);
  await startGame(page, { seed: "1", first: "a", deckA: "basic-forest", deckB: "basic-rune" });
  await confirmMulligans(page);
  await page.locator("#endTurnBlue:visible").click();
  await expect(page.locator("#redBoostHost #bonusPpBtn")).toBeEnabled();
  await mkdir(ART, { recursive: true });
  await artShot(page.locator("#turnControls"), `${ART}/boost_on_red_second.png`);

  const paints: number[] = [];
  const playable = page.locator("#redHand .card.legal-play");
  if (await playable.count()) {
    await playable.first().click();
    paints.push(await page.evaluate(() => window.__arena?.paintMs ?? -1));
  }
  await page.locator("#endTurnRed:visible").click();
  paints.push(await page.evaluate(() => window.__arena?.paintMs ?? -1));
  console.log(`paintMs after actions: ${JSON.stringify(paints)}`);
  expect(paints.every((n) => n >= 0 && n < 80)).toBeTruthy();
});
