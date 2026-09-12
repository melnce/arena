import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
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

async function assertHiddenBotHand(page: Page) {
  const snap = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { hand: unknown[] }; b: { hand: unknown[] } };
    };
    const zone = document.getElementById("redHand");
    const cards = zone ? Array.from(zone.querySelectorAll<HTMLElement>(":scope > .card")) : [];
    return {
      engine: full.players.b.hand.length,
      rail: document.getElementById("redHandCount")?.textContent ?? "",
      children: cards.length,
      backs: cards.filter((c) => c.classList.contains("card-back")).length,
      leaks: cards.map((c) => ({
        card: c.dataset.card ?? "",
        name: c.dataset.name ?? "",
        cost: c.querySelector(".cost-badge")?.textContent ?? "",
      })),
    };
  });
  expect(snap.children).toBe(snap.engine);
  expect(snap.backs).toBe(snap.engine);
  expect(snap.rail).toBe(String(snap.engine));
  for (const leak of snap.leaks) {
    expect(leak.card).toBe("");
    expect(leak.name).toBe("");
    expect(leak.cost).toBe("");
  }
  return snap;
}

test("vs-bot hidden hand matches engine count and stays leak-free", async ({ page }) => {
  test.setTimeout(90_000);
  await page.setViewportSize({ width: 1280, height: 720 });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#hideBotHandToggle").check();
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#humanSideSelect").selectOption("a");
  await page.locator("#vsBotPolicy").selectOption("h0");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  await assertHiddenBotHand(page);

  await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const play = legal.find((a) => "play" in a);
    if (play) window.__arena!.apply(play);
    const choose = (window.__arena!.legal() as Array<Record<string, unknown>>).find(
      (a) => "choose" in a || "confirm" in a,
    );
    if (choose) window.__arena!.apply(choose);
  });
  const end = page.locator("#endTurnBlue:visible, #endTurnRed:visible");
  await expect(end).toHaveCount(1, { timeout: 8000 });
  await end.first().click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 20_000,
  });
  const after = await assertHiddenBotHand(page);
  expect(after.engine).toBeGreaterThan(0);

  const zone = page.locator("#redHand");
  await artShot(zone, `${ART}/hidden_hand.png`);
  await artShot(zone, "/tmp/hidden_hand.png");
  const colors = await zone.locator(".card.card-back").first().evaluate((el) => {
    const zoneEl = el.parentElement as HTMLElement;
    return {
      border: getComputedStyle(el).borderColor,
      zone: getComputedStyle(zoneEl).backgroundColor,
    };
  });
  expect(colors.border).not.toBe(colors.zone);
  expect(colors.border).not.toBe("rgba(0, 0, 0, 0)");

  await page.setViewportSize({ width: 1024, height: 768 });
  await assertHiddenBotHand(page);
});
