import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

async function boot(page: Page, query = "") {
  await page.goto(`/${query ? `?${query}` : ""}`);
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

test("bot speed test bench", async ({ page }) => {
  await boot(page, "benchspec=h0:nodes=200");
  await openSettings(page);
  const gen0 = await page.evaluate(() => window.__arena!.botWorker().generation);

  await page.locator("#benchRunBtn").click();
  await expect(page.locator("#benchOutput")).toContainText("fingerprint:", { timeout: 120_000 });
  const text = (await page.locator("#benchOutput").textContent()) ?? "";
  const rowCount = text.split("\n").filter((l) => /^\d+\t/.test(l)).length;
  expect(rowCount).toBe(10);
  expect(text).toMatch(/median:/);
  expect(text).toMatch(/fingerprint:/);

  const fp1 = text.match(/fingerprint: ([0-9a-f]+)/)?.[1];
  expect(fp1).toBeTruthy();

  await page.locator("#benchRunBtn").click();
  await expect(page.locator("#benchOutput")).toContainText("fingerprint:", { timeout: 120_000 });
  const text2 = (await page.locator("#benchOutput").textContent()) ?? "";
  const fp2 = text2.match(/fingerprint: ([0-9a-f]+)/)?.[1];
  expect(fp2).toBe(fp1);

  await page.locator("#benchRunBtn").click();
  await page.waitForTimeout(300);
  await page.locator("#benchStopBtn").click();
  await page.waitForTimeout(500);
  const gen1 = await page.evaluate(() => window.__arena!.botWorker().generation);
  expect(gen1).toBe(gen0);

  const json = await page.evaluate(() => window.__arena!.benchResult());
  expect(json).toBeTruthy();
  await page.locator("#benchCopyBtn").click();
});
