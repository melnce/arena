import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import {
  CUSTOM_SPEC_MENU_VALUE,
  resolveStoredBotPolicy,
  STORED_SERVED_CHEAT,
  STORED_SERVED_STRONG,
} from "../src/policy-labels.ts";
import { openSettings } from "./helpers.ts";

const STRONG_H0 = "h0:nodes=6000";
const CHEAT_H0 = "h0:nodes=6000,info=all";
const SUNDAY_STRONG =
  "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";
const SUNDAY_CHEAT =
  "h0:nodes=32000,horizon=3,k=8,info=all,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";

function readPcPolicies(): { strong: string; cheat: string } {
  const servePath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../py/serve.py");
  const text = fs.readFileSync(servePath, "utf8");
  const strongM = text.match(/^DEFAULT_STRONG = "(.*)"$/m);
  const cheatM = text.match(/^DEFAULT_CHEAT = "(.*)"$/m);
  if (!strongM || !cheatM) throw new Error("serve.py policies missing");
  return { strong: strongM[1], cheat: cheatM[1] };
}

const PC = readPcPolicies();

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function openVsBot(page: Page) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
}

test.describe("resolveStoredBotPolicy", () => {
  const servedStrong = PC.strong;
  const servedCheat = PC.cheat;
  const allowed = new Set([
    "random",
    "first-legal",
    "h0",
    STRONG_H0,
    CHEAT_H0,
    servedStrong,
    servedCheat,
    CUSTOM_SPEC_MENU_VALUE,
  ]);
  const fallbackDesktop = STRONG_H0;
  const fallbackTablet = "h0";

  const cases: Array<{ stored: string | null; fallback: string; want: string }> = [
    { stored: STORED_SERVED_STRONG, fallback: fallbackTablet, want: servedStrong },
    { stored: STORED_SERVED_CHEAT, fallback: fallbackTablet, want: servedCheat },
    { stored: "h0", fallback: fallbackDesktop, want: "h0" },
    { stored: STRONG_H0, fallback: fallbackTablet, want: STRONG_H0 },
    { stored: servedStrong, fallback: fallbackTablet, want: servedStrong },
    { stored: SUNDAY_STRONG, fallback: fallbackTablet, want: servedStrong },
    { stored: SUNDAY_CHEAT, fallback: fallbackDesktop, want: servedCheat },
    { stored: "h0:nodes=999999999", fallback: fallbackDesktop, want: fallbackDesktop },
    { stored: "h0:nodes=4000", fallback: fallbackTablet, want: fallbackTablet },
    { stored: null, fallback: fallbackDesktop, want: fallbackDesktop },
  ];

  for (const { stored, fallback, want } of cases) {
    test(`stored=${stored ?? "null"} → ${want === servedStrong ? "servedStrong" : want === servedCheat ? "servedCheat" : want}`, () => {
      expect(
        resolveStoredBotPolicy(stored, allowed, servedStrong, servedCheat, fallback),
      ).toBe(want);
    });
  }
});

test.describe("tablet owner: former served spec", () => {
  test.use({ hasTouch: true, isMobile: true, viewport: { width: 1180, height: 820 } });

  test("Sunday Full strength → current DEFAULT_STRONG", async ({ page }) => {
    await page.addInitScript((spec) => {
      localStorage.setItem("svwb.botPolicy", spec);
    }, SUNDAY_STRONG);
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.strong);
    const label = await page.locator("#vsBotPolicy option:checked").textContent();
    expect(label?.startsWith("Full strength")).toBe(true);
  });

  test("missing svwb.botPolicy still defaults to Quick (h0)", async ({ page }) => {
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue("h0");
  });
});

test.describe("desktop: former served spec", () => {
  test("Sunday Full strength → current DEFAULT_STRONG", async ({ page }) => {
    await page.addInitScript((spec) => {
      localStorage.setItem("svwb.botPolicy", spec);
    }, SUNDAY_STRONG);
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.strong);
    const label = await page.locator("#vsBotPolicy option:checked").textContent();
    expect(label?.startsWith("Full strength")).toBe(true);
  });

  test("Sunday cheater spec → current DEFAULT_CHEAT", async ({ page }) => {
    await page.addInitScript((spec) => {
      localStorage.setItem("svwb.botPolicy", spec);
    }, SUNDAY_CHEAT);
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.cheat);
    const label = await page.locator("#vsBotPolicy option:checked").textContent();
    expect(label?.startsWith("Full strength, sees your hand")).toBe(true);
  });
});

test("choosing Full strength stores served:strong; reload selects DEFAULT_STRONG", async ({
  page,
}) => {
  await boot(page);
  await openVsBot(page);
  await page.locator("#vsBotPolicy").selectOption(PC.strong);
  const stored = await page.evaluate(() => localStorage.getItem("svwb.botPolicy"));
  expect(stored).toBe(STORED_SERVED_STRONG);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.strong);
});

test("choosing cheat Full strength stores served:cheat; reload selects DEFAULT_CHEAT", async ({
  page,
}) => {
  await boot(page);
  await openVsBot(page);
  await page.locator("#vsBotPolicy").selectOption(PC.cheat);
  const stored = await page.evaluate(() => localStorage.getItem("svwb.botPolicy"));
  expect(stored).toBe(STORED_SERVED_CHEAT);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.cheat);
});

test("init script served:strong / served:cheat select current specs", async ({ page }) => {
  await page.addInitScript((strongKey) => {
    localStorage.setItem("svwb.botPolicy", strongKey);
  }, STORED_SERVED_STRONG);
  await boot(page);
  await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.strong);

  await page.addInitScript((cheatKey) => {
    localStorage.setItem("svwb.botPolicy", cheatKey);
  }, STORED_SERVED_CHEAT);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(PC.cheat);
});

test("custom spec and Quick/Medium persist as plain values", async ({ page }) => {
  const customSpec =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";
  await boot(page);
  await openVsBot(page);
  await page.locator("#vsBotPolicy").selectOption("h0");
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue("h0");

  await openVsBot(page);
  await page.locator("#vsBotPolicy").selectOption(STRONG_H0);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(STRONG_H0);

  await page.evaluate((spec) => {
    localStorage.setItem("svwb.customBotSpec", spec);
    localStorage.setItem("svwb.botPolicy", spec);
  }, customSpec);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(customSpec);
  const label = await page.locator("#vsBotPolicy option:checked").textContent();
  expect(label).toContain("Custom:");
});
