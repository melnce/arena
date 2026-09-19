import { expect, test, devices, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const STRONG = "h0:nodes=6000";

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function startGame(
  page: Page,
  opts: {
    mode: string;
    seed?: string;
    first?: string;
    deckA?: string;
    deckB?: string;
    human?: string;
    botPolicy?: string;
  },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode);
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
  if (opts.human) await page.locator("#humanSideSelect").selectOption(opts.human);
  if (opts.botPolicy) {
    await page.locator("#vsBotPolicy").selectOption(opts.botPolicy);
    const human = opts.human ?? "a";
    const other = human === "b" ? "#policyASelect" : "#policyBSelect";
    await page.locator(other).selectOption(opts.botPolicy, { force: true });
  }
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

async function vsBotOptions(page: Page) {
  return page.locator("#vsBotPolicy option").evaluateAll((opts) =>
    opts.map((el) => {
      const o = el as HTMLOptionElement;
      return { value: o.value, label: o.textContent ?? "", title: o.title };
    }),
  );
}

test("desktop: option list defaults to h0 (strong)", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  const opts = await vsBotOptions(page);
  expect(opts.map((o) => o.value)).toEqual(["random", "first-legal", "h0", STRONG]);
  expect(opts[2]?.label).toBe("h0 (standard)");
  expect(opts[3]?.label).toBe("h0 (strong)");
  expect(opts[3]?.title).toContain("6 000 search nodes");
  await expect(page.locator("#vsBotPolicy")).toHaveValue(STRONG);
  await expect(page.locator("#vsBotPolicyHint")).toContainText("6 000 search nodes");
  await page.locator("#vsBotPolicy").evaluate((el) => {
    (el as HTMLSelectElement).size = 4;
  });
  await artShot(page.locator("#vsBotFields"), `${ART}/bot_strength_options.png`);
});

test.describe("phone", () => {
  const pixel5 = devices["Pixel 5"];
  test.use({
    viewport: pixel5.viewport,
    userAgent: pixel5.userAgent,
    deviceScaleFactor: pixel5.deviceScaleFactor,
    isMobile: pixel5.isMobile,
    hasTouch: pixel5.hasTouch,
  });

  test("defaults to standard h0", async ({ page }) => {
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue("h0");
  });
});

test("persistence: stored choice survives reload; bogus falls back", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").selectOption("h0");
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue("h0");

  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").selectOption(STRONG);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(STRONG);
});

for (const stored of ["h0:nodes=999999999", "h0:nodes=4000"] as const) {
  test(`bogus stored policy falls back to the device default (${stored})`, async ({ page }) => {
    await page.addInitScript((value) => {
      localStorage.setItem("svwb.botPolicy", value);
    }, stored);
    await boot(page);
    await expect(page.locator("#vsBotPolicy")).toHaveValue(STRONG);
  });
}

test("vs bot: human A vs h0:nodes=6000, bot turn resolves", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: STRONG,
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 30_000,
  });
  await expect(page.locator("#eventLog")).not.toHaveText("", { timeout: 15_000 });
  const events = await page.locator("#eventLog").innerText();
  expect(events.length).toBeGreaterThan(0);
  const ms = await page.evaluate(() => {
    const t0 = performance.now();
    window.__arena!.botAction("h0:nodes=6000", "1");
    return performance.now() - t0;
  });
  console.log(`strong botAction ${ms.toFixed(1)} ms`);
});

test("vs-bot coin: both seats get vsBotPolicy, not leftover random", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#humanSideSelect").selectOption("coin");
  await page.locator("#seedInput").fill("7");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#policyASelect").selectOption("random", { force: true });
  await page.locator("#policyBSelect").selectOption("random", { force: true });
  await page.locator("#vsBotPolicy").selectOption(STRONG);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  const snap = await page.evaluate(() => ({
    policies: window.__arena!.policies(),
    human: window.__arena!.humanSide(),
  }));
  expect(snap.policies).toEqual([STRONG, STRONG]);
  expect(snap.human === "a" || snap.human === "b").toBeTruthy();
});
