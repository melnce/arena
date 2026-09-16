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

async function mockLocalBot(
  page: Page,
  opts?: { failBotOnce?: boolean; onBot?: () => void },
): Promise<{ botRequests: () => number }> {
  let botRequests = 0;
  let failed = false;
  await page.route("http://127.0.0.1:8765/**", async (route) => {
    const url = route.request().url();
    if (url.includes("/health")) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          ok: true,
          strong: "h0:nodes=16000",
          cpus: 8,
          version: "test",
        }),
      });
      return;
    }
    if (url.includes("/bot")) {
      botRequests += 1;
      opts?.onBot?.();
      if (opts?.failBotOnce && !failed) {
        failed = true;
        await route.fulfill({
          status: 500,
          contentType: "application/json",
          body: JSON.stringify({ error: "synthetic server error" }),
        });
        return;
      }
      const posted = route.request().postDataJSON() as { botSeed?: string; hash?: string };
      const seed = posted.botSeed ?? "1";
      const actionJson = await page.evaluate(
        (s) => window.__arena!.botAction("h0", s),
        seed,
      );
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          action: JSON.parse(actionJson),
          policy: "h0:nodes=16000",
          ms: 1,
          hash: posted.hash ?? "",
        }),
      });
      return;
    }
    await route.fallback();
  });
  return { botRequests: () => botRequests };
}

test("no server: badge is browser and vs-bot still plays", async ({ page }) => {
  await boot(page);
  await expect(page.locator("#botBackendBadge")).toHaveText("bot: browser");
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  await expect(page.locator("#eventLog")).not.toHaveText("", { timeout: 15_000 });
  const state = await page.evaluate(() => window.__arena!.localBot());
  expect(state.backend).toBe("browser");
  expect(state.remote).toBe(0);
});

test("mocked server: badge, remote step, event log", async ({ page }) => {
  const mock = await mockLocalBot(page);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await expect(page.locator("#botBackendBadge")).toHaveText(
    "bot: local server (h0:nodes=16000, 8 cpus)",
    { timeout: 5_000 },
  );
  await artShot(page.locator("#vsBotFields"), `${ART}/local_bot_badge.png`);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  await expect(page.locator("#eventLog")).not.toHaveText("", { timeout: 15_000 });
  const state = await page.evaluate(() => window.__arena!.localBot());
  expect(state.remote).toBeGreaterThanOrEqual(1);
  expect(state.backend).toBe("server");
  expect(mock.botRequests()).toBeGreaterThanOrEqual(1);
});

test("server 500 falls back to wasm and pins the error badge", async ({ page }) => {
  await mockLocalBot(page, { failBotOnce: true });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await expect(page.locator("#botBackendBadge")).toHaveText(
    "bot: local server (h0:nodes=16000, 8 cpus)",
    { timeout: 5_000 },
  );
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  await expect(page.locator("#botBackendBadge")).toContainText("bot: browser (server error:");
  const state = await page.evaluate(() => window.__arena!.localBot());
  expect(state.backend).toBe("browser");
  expect(state.error).toBeTruthy();
  const n = await page.evaluate(() => window.__arena!.actions().length);
  expect(n).toBeGreaterThan(0);
});

test("settings toggle off: no /bot even with a mock", async ({ page }) => {
  const mock = await mockLocalBot(page);
  await boot(page);
  await openSettings(page);
  await page.locator("#localBotToggle").uncheck();
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  expect(mock.botRequests()).toBe(0);
  await expect(page.locator("#botBackendBadge")).toHaveText("bot: browser");
});

test("slow /health still becomes local server", async ({ page }) => {
  await page.route("http://127.0.0.1:8765/**", async (route) => {
    const url = route.request().url();
    if (url.includes("/health")) {
      await new Promise((r) => setTimeout(r, 1000));
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          ok: true,
          strong: "h0:nodes=16000",
          cpus: 8,
          version: "test",
        }),
      });
      return;
    }
    await route.fallback();
  });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await expect(page.locator("#botBackendBadge")).toHaveText(
    "bot: local server (h0:nodes=16000, 8 cpus)",
    { timeout: 5_000 },
  );
});

test("stale /bot after a new game is dropped", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (err) => pageErrors.push(String(err)));

  let firstHeld = false;
  let releaseFirst: ((action: Record<string, unknown>) => void) | null = null;
  const later: Array<{ seed: string; hash: string }> = [];

  await page.route("http://127.0.0.1:8765/**", async (route) => {
    const url = route.request().url();
    if (url.includes("/health")) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          ok: true,
          strong: "h0:nodes=16000",
          cpus: 8,
          version: "test",
        }),
      });
      return;
    }
    if (url.includes("/bot")) {
      const posted = route.request().postDataJSON() as {
        seed?: string;
        botSeed?: string;
        hash?: string;
      };
      if (!firstHeld) {
        firstHeld = true;
        const action = await new Promise<Record<string, unknown>>((resolve) => {
          releaseFirst = resolve;
        });
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            action,
            policy: "h0:nodes=16000",
            ms: 1,
            hash: posted.hash ?? "",
          }),
        });
        return;
      }
      later.push({ seed: String(posted.seed ?? ""), hash: String(posted.hash ?? "") });
      const actionJson = await page.evaluate(
        (s) => window.__arena!.botAction("h0", s),
        posted.botSeed ?? "1",
      );
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          action: JSON.parse(actionJson),
          policy: "h0:nodes=16000",
          ms: 1,
          hash: posted.hash ?? "",
        }),
      });
      return;
    }
    await route.fallback();
  });

  await boot(page);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  await expect.poll(() => firstHeld).toBe(true);

  await startGame(page, {
    mode: "vs-bot",
    seed: "99",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  releaseFirst?.({ mulligan: { player: "b", swap: [false, false, false, false] } });

  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  expect(pageErrors).toEqual([]);
  expect(later.every((r) => r.seed === "99")).toBe(true);
  const state = await page.evaluate(() => window.__arena!.localBot());
  expect(state.remote).toBe(later.length);
  await expect(page.locator("#botBackendBadge")).not.toContainText("thinking");
});

test("stale /bot after undo is dropped", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (err) => pageErrors.push(String(err)));

  let firstHeld = false;
  let releaseFirst: ((action: Record<string, unknown>) => void) | null = null;

  await page.route("http://127.0.0.1:8765/**", async (route) => {
    const url = route.request().url();
    if (url.includes("/health")) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          ok: true,
          strong: "h0:nodes=16000",
          cpus: 8,
          version: "test",
        }),
      });
      return;
    }
    if (url.includes("/bot")) {
      const posted = route.request().postDataJSON() as { botSeed?: string; hash?: string };
      if (!firstHeld) {
        firstHeld = true;
        const action = await new Promise<Record<string, unknown>>((resolve) => {
          releaseFirst = resolve;
        });
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            action,
            policy: "h0:nodes=16000",
            ms: 1,
            hash: posted.hash ?? "",
          }),
        });
        return;
      }
      const actionJson = await page.evaluate(
        (s) => window.__arena!.botAction("h0", s),
        posted.botSeed ?? "1",
      );
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          action: JSON.parse(actionJson),
          policy: "h0:nodes=16000",
          ms: 1,
          hash: posted.hash ?? "",
        }),
      });
      return;
    }
    await route.fallback();
  });

  await boot(page);
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "h0",
  });
  const humanConfirm = page.locator("#blueMulliganConfirm").locator("visible=true");
  await expect(humanConfirm).toBeVisible({ timeout: 5_000 });
  await humanConfirm.click();
  await expect.poll(() => firstHeld).toBe(true);

  await openSettings(page);
  await expect(page.locator("#undoBtn")).toBeEnabled();
  const beforeUndo = await page.evaluate(() => window.__arena!.actions().length);
  await page.locator("#undoBtn").click();
  const afterUndo = await page.evaluate(() => window.__arena!.actions().length);
  expect(afterUndo).toBeLessThan(beforeUndo);

  releaseFirst?.({ mulligan: { player: "b", swap: [false, false, false, false] } });
  await page.waitForTimeout(400);
  expect(await page.evaluate(() => window.__arena!.actions().length)).toBe(afterUndo);
  expect(pageErrors).toEqual([]);
  await expect(page.locator("#botBackendBadge")).not.toContainText("thinking");
});

test("first-legal vs-bot policy never POSTs /bot", async ({ page }) => {
  const mock = await mockLocalBot(page);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await expect(page.locator("#botBackendBadge")).toHaveText(
    "bot: local server (h0:nodes=16000, 8 cpus)",
    { timeout: 5_000 },
  );
  await startGame(page, {
    mode: "vs-bot",
    seed: "1",
    first: "b",
    deckA: "basic-forest",
    deckB: "basic-rune",
    human: "a",
    botPolicy: "first-legal",
  });
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", {
    timeout: 15_000,
  });
  expect(mock.botRequests()).toBe(0);
});
