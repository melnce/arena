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

async function setWatchSpeed(page: Page, value: number) {
  await page.locator("#watchSpeed").evaluate((el, v) => {
    const input = el as HTMLInputElement;
    input.value = String(v);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }, value);
}

test("watchDelayMs curve, readout, persist, and slow-end spacing", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await expect(page.locator("#watchSpeed")).toHaveValue("5");
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(938);
  await expect(page.locator("#watchSpeedReadout")).toHaveText("0.94 s / action");

  await setWatchSpeed(page, 1);
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(3000);
  await expect(page.locator("#watchSpeedReadout")).toHaveText("3.0 s / action");

  await setWatchSpeed(page, 20);
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(0);
  await expect(page.locator("#watchSpeedReadout")).toHaveText("max");

  await setWatchSpeed(page, 10);
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(219);
  await expect(page.locator("#watchSpeedReadout")).toHaveText("0.22 s / action");
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#watchSpeed")).toHaveValue("10");
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(219);
  await expect(page.locator("#watchSpeedReadout")).toHaveText("0.22 s / action");

  await setWatchSpeed(page, 1);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("watch");
  await page.locator("#watchAutoStart").uncheck();
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#policyASelect").selectOption("h0", { force: true });
  await page.locator("#policyBSelect").selectOption("random", { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /.+/, {
    timeout: 15_000,
  });
  expect(await page.evaluate(() => window.__arena!.watchDelayMs())).toBe(3000);
  await expect(page.locator("#watchBar")).toBeVisible();
  await artShot(page.locator("#watchBar"), `${ART}/watch_speed_readout.png`);

  const h0 = await page.evaluate(() => window.__arena!.hash());
  await page.locator("#watchPlayBtn").click();
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).not.toBe(h0);
  const t1 = Date.now();
  const h1 = await page.evaluate(() => window.__arena!.hash());
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).not.toBe(h1);
  const gap = Date.now() - t1;
  expect(gap).toBeGreaterThanOrEqual(2500);
});
