import { expect, test, type Page } from "@playwright/test";
import { openSettings } from "./helpers.ts";

const ONE_COST = "10052110";
const CAPTURE_KEY = "test-key-123";

async function boot(page: Page, hash = "") {
  await page.goto(`/${hash}`);
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

async function startVsBot(
  page: Page,
  opts: {
    seed: string;
    humanDeck: string;
    botDeck: string;
    human?: "a" | "b";
    first?: "a" | "b" | "coin";
    botPolicy?: string;
  },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill(opts.seed);
  await page.locator("#firstSelect").selectOption(opts.first ?? "a");
  await page.locator("#humanSideSelect").selectOption(opts.human ?? "a");
  await page.locator("#blueDeckSelect").selectOption(opts.humanDeck);
  await page.locator("#redDeckSelect").selectOption(opts.botDeck);
  const policy = opts.botPolicy ?? "first-legal";
  await page.locator("#vsBotPolicy").selectOption(policy);
  const human = opts.human ?? "a";
  const other = human === "b" ? "#policyASelect" : "#policyBSelect";
  await page.locator(other).selectOption(policy, { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmHumanMulligan(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "#blueMulliganConfirm" : "#redMulliganConfirm";
  const btn = page.locator(id);
  await expect(btn).toBeVisible({ timeout: 15_000 });
  await btn.click();
  await expect(btn).toBeHidden({ timeout: 5000 });
}

async function playToTerminal(page: Page) {
  for (let i = 0; i < 120; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
      const act =
        legal.find((a) => "end_turn" in a) ??
        legal.find((a) => "play" in a) ??
        legal[0];
      if (act) window.__arena!.apply(act);
    });
    await page.waitForTimeout(200);
  }
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "terminal", { timeout: 60_000 });
}

test("capture key from hash link", async ({ page }) => {
  await boot(page, "#capturekey=test-key-123");
  const stored = await page.evaluate(() => localStorage.getItem("svwb.captureKey"));
  expect(stored).toBe("test-key-123");
  expect(page.url()).not.toContain("capturekey");
});

test("finished vs-bot game uploads once with final capture", async ({ page }) => {
  test.setTimeout(120_000);
  const posts: Array<{ headers: Record<string, string>; body: Record<string, unknown> }> = [];
  await page.route("**/api/capture", async (route) => {
    const req = route.request();
    posts.push({
      headers: req.headers(),
      body: req.postDataJSON() as Record<string, unknown>,
    });
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ ok: true }) });
  });
  await boot(page, `#capturekey=${CAPTURE_KEY}`);
  await page.goto("/?localbot=0");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });

  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap-human.json", deck);
  const botDeck = await importDeck(page, "cap-bot.json", deck);
  await startVsBot(page, { seed: "9001", humanDeck, botDeck, first: "a", botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  await expect.poll(() => posts.length).toBe(1);
  const body = posts[0]!.body;
  expect(body.final).toBe(true);
  // cd py && python3 -c "from serve import make_game_id; print(make_game_id(9001, {'10052110': 8}, {'10052110': 8}, 'a'))"
  expect(body.game_id).toBe("9001-d929b0fc");
  expect(body.actions).toEqual(await page.evaluate(() => window.__arena!.actions()));
  expect(body.humanSide).toBe("a");
  expect(body.winner).toBe(
    await page.evaluate(() => (window.__arena!.full() as { winner: string | null }).winner),
  );
  expect(posts[0]!.headers["x-capture-key"]).toBe(CAPTURE_KEY);
});

test("unfinished game stored as partial when new session starts", async ({ page }) => {
  const posts: Array<Record<string, unknown>> = [];
  await page.route("**/api/capture", async (route) => {
    posts.push(route.request().postDataJSON() as Record<string, unknown>);
    await route.fulfill({ status: 200, body: JSON.stringify({ ok: true }) });
  });
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 40 };
  const humanDeck = await importDeck(page, "cap2-human.json", deck);
  const botDeck = await importDeck(page, "cap2-bot.json", deck);
  await startVsBot(page, { seed: "77", humanDeck, botDeck, botPolicy: "first-legal", first: "b" });
  await confirmHumanMulligan(page, "a");
  await endHumanTurn(page, "a");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 30_000 });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 60_000 });
  const session1 = await page.evaluate(() => {
    const caps = window.__arena!.captures();
    return caps;
  });
  await startVsBot(page, { seed: "88", humanDeck, botDeck, botPolicy: "first-legal", first: "a" });
  await expect.poll(() => posts.filter((p) => p.final === false).length, { timeout: 20_000 }).toBeGreaterThanOrEqual(1);
  const partial = posts.find((p) => p.final === false);
  expect(partial?.session_id).toBeTruthy();
  await confirmHumanMulligan(page, "a");
  await endHumanTurn(page, "a");
  await startVsBot(page, { seed: "99", humanDeck, botDeck, botPolicy: "first-legal", first: "a" });
  await expect.poll(() => posts.length, { timeout: 20_000 }).toBeGreaterThanOrEqual(2);
  const sessionIds = posts.map((p) => p.session_id as string);
  expect(new Set(sessionIds).size).toBeGreaterThanOrEqual(2);
  expect(session1.waiting).toBeGreaterThanOrEqual(0);
});

test("offline queue then upload now", async ({ page }) => {
  test.setTimeout(120_000);
  let fail = true;
  await page.route("**/api/capture", async (route) => {
    if (fail) {
      await route.fulfill({ status: 503, body: JSON.stringify({ error: "uploads not configured" }) });
      return;
    }
    await route.fulfill({ status: 200, body: JSON.stringify({ ok: true }) });
  });
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap3-human.json", deck);
  const botDeck = await importDeck(page, "cap3-bot.json", deck);
  await startVsBot(page, { seed: "303", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  await expect.poll(async () => (await page.evaluate(() => window.__arena!.captures())).waiting).toBe(1);
  fail = false;
  await openSettings(page);
  await page.locator("#captureUploadNowBtn").click();
  await expect.poll(async () => (await page.evaluate(() => window.__arena!.captures())).waiting).toBe(0);
});

test("401 shows upload key rejected and stops posts", async ({ page }) => {
  test.setTimeout(120_000);
  let count = 0;
  await page.route("**/api/capture", async (route) => {
    count += 1;
    await route.fulfill({ status: 401, body: JSON.stringify({ error: "unauthorized" }) });
  });
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap4-human.json", deck);
  const botDeck = await importDeck(page, "cap4-bot.json", deck);
  await startVsBot(page, { seed: "404", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  await expect.poll(() => count).toBe(1);
  await openSettings(page);
  await expect(page.locator("#captureUploadStatus")).toContainText("upload key rejected");
  await page.locator("#captureUploadNowBtn").click();
  expect(count).toBe(1);
});

test("no capture key means no posts during vs-bot game", async ({ page }) => {
  test.setTimeout(120_000);
  let count = 0;
  await page.route("**/api/capture", async (route) => {
    count += 1;
    await route.fulfill({ status: 200, body: "{}" });
  });
  await boot(page, "?localbot=0");
  await page.evaluate(() => localStorage.removeItem("svwb.captureKey"));
  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap5-human.json", deck);
  const botDeck = await importDeck(page, "cap5-bot.json", deck);
  await startVsBot(page, { seed: "505", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  expect(count).toBe(0);
});

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

test("bot_meta worker and server value", async ({ page }) => {
  test.setTimeout(90_000);
  let botHits = 0;
  await page.route("**/127.0.0.1:8765/**", async (route) => {
    const url = route.request().url();
    if (url.includes("/health")) {
      await route.fulfill({
        status: 200,
        body: JSON.stringify({ ok: true, strong: "h0:nodes=1", cpus: 4, version: "t" }),
      });
      return;
    }
    if (url.includes("/bot")) {
      botHits += 1;
      const posted = route.request().postDataJSON() as { botSeed?: string };
      const actionJson = await page.evaluate((s) => window.__arena!.botAction("h0", s), posted.botSeed ?? "1");
      await route.fulfill({
        status: 200,
        body: JSON.stringify({ action: JSON.parse(actionJson), ms: 12, value: 12.5 }),
      });
      return;
    }
    if (url.includes("/game")) {
      await route.fulfill({ status: 200, body: JSON.stringify({ ok: true }) });
      return;
    }
    await route.continue();
  });
  await boot(page, `?botworker=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 40 };
  const humanDeck = await importDeck(page, "cap6-human.json", deck);
  const botDeck = await importDeck(page, "cap6-bot.json", deck);
  await startVsBot(page, { seed: "606", humanDeck, botDeck, botPolicy: "h0", first: "a" });
  await confirmHumanMulligan(page, "a");
  await expect
    .poll(async () => page.evaluate(() => window.__arena!.localBot().backend), { timeout: 15_000 })
    .toBe("server");
  await endHumanTurn(page, "a");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 60_000 });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 120_000 });
  expect(botHits).toBeGreaterThan(0);
  await expect
    .poll(async () =>
      page.evaluate(() =>
        window.__arena!.captureBotMeta().some((m) => m.backend === "server" && m.value === 12.5),
      ),
    )
    .toBe(true);
  await page.evaluate(async () => {
    await window.__arena!.storeCaptureNow();
  });
  const replayOk = await page.evaluate(() => {
    const actions = window.__arena!.actions() as Array<Record<string, unknown>>;
    if (actions.some((a) => "bot_value" in a)) return false;
    window.__arena!.loadLog(window.__arena!.exportPositionLog());
    return window.__arena!.hash().length > 0;
  });
  expect(replayOk).toBe(true);
  const stored = await page.evaluate(async () => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const req = indexedDB.open("arena-captures");
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
    const all = await new Promise<Array<{ capture?: { actions?: unknown[]; bot_meta?: unknown[] } }>>(
      (resolve, reject) => {
        const tx = db.transaction("captures", "readonly");
        const r = tx.objectStore("captures").getAll();
        r.onsuccess = () => resolve(r.result as Array<{ capture?: { actions?: unknown[]; bot_meta?: unknown[] } }>);
        r.onerror = () => reject(r.error);
      },
    );
    db.close();
    const rec = all.find((row) =>
      (row.capture?.actions as Array<Record<string, unknown>> | undefined)?.some(
        (a) => a.bot_value === 12.5,
      ),
    );
    return {
      withValue: Boolean(rec),
      actions: (rec?.capture?.actions ?? []) as Array<Record<string, unknown>>,
      bot_meta: (rec?.capture?.bot_meta ?? []) as Array<{ backend: string; value: number | null }>,
    };
  });
  expect(stored.withValue).toBe(true);
  expect(stored.bot_meta.some((m) => m.backend === "server" && m.value === 12.5)).toBe(true);
});

test("bot_meta worker ms with h0 in browser", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 40 };
  const humanDeck = await importDeck(page, "cap7-human.json", deck);
  const botDeck = await importDeck(page, "cap7-bot.json", deck);
  await startVsBot(page, { seed: "707", humanDeck, botDeck, botPolicy: "h0", first: "a" });
  await confirmHumanMulligan(page, "a");
  await endHumanTurn(page, "a");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 60_000 });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 120_000 });
  await startVsBot(page, { seed: "708", humanDeck, botDeck, botPolicy: "h0", first: "a" });
  const workerMeta = await page.evaluate(async () => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const req = indexedDB.open("arena-captures");
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
    const all = await new Promise<{ capture: { bot_meta: Array<{ backend: string; ms: number | null }> } }[]>(
      (resolve, reject) => {
        const tx = db.transaction("captures", "readonly");
        const r = tx.objectStore("captures").getAll();
        r.onsuccess = () =>
          resolve(r.result as { capture: { bot_meta: Array<{ backend: string; ms: number | null }> } }[]);
        r.onerror = () => reject(r.error);
      },
    );
    db.close();
    const cap = all[all.length - 1]?.capture;
    return cap?.bot_meta ?? [];
  });
  const workers = workerMeta.filter((m) => m.backend === "worker");
  expect(workers.length).toBeGreaterThan(0);
  expect(workers.every((m) => typeof m.ms === "number" && m.ms >= 0)).toBe(true);
});

test("visibility hidden after terminal keeps final capture on upload", async ({ page }) => {
  test.setTimeout(120_000);
  let fail = true;
  const posts: Array<Record<string, unknown>> = [];
  await page.route("**/api/capture", async (route) => {
    posts.push(route.request().postDataJSON() as Record<string, unknown>);
    if (fail) {
      await route.fulfill({ status: 503, body: JSON.stringify({ error: "uploads not configured" }) });
      return;
    }
    await route.fulfill({ status: 200, body: JSON.stringify({ ok: true }) });
  });
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap-vis-human.json", deck);
  const botDeck = await importDeck(page, "cap-vis-bot.json", deck);
  await startVsBot(page, { seed: "909", humanDeck, botDeck, botPolicy: "first-legal", first: "a" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  const winner = await page.evaluate(
    () => (window.__arena!.full() as { winner: string | null }).winner,
  );
  fail = false;
  await page.evaluate(() => {
    Object.defineProperty(document, "visibilityState", {
      configurable: true,
      get: () => "hidden",
    });
    document.dispatchEvent(new Event("visibilitychange"));
  });
  await openSettings(page);
  await page.locator("#captureUploadNowBtn").click();
  await expect.poll(() => posts.length, { timeout: 15_000 }).toBeGreaterThanOrEqual(1);
  expect(posts.every((p) => p.final === true)).toBe(true);
  expect(posts.some((p) => p.final !== true)).toBe(false);
  expect(posts[0]!.winner).toBe(winner);
});

test("export games downloads captures array", async ({ page }) => {
  await boot(page, `?localbot=0#capturekey=${CAPTURE_KEY}`);
  const deck = { [ONE_COST]: 8 };
  const humanDeck = await importDeck(page, "cap8-human.json", deck);
  const botDeck = await importDeck(page, "cap8-bot.json", deck);
  await startVsBot(page, { seed: "808", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await playToTerminal(page);
  await openSettings(page);
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.locator("#captureExportBtn").click(),
  ]);
  const path = await download.path();
  expect(path).toBeTruthy();
  const text = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const c of text!) chunks.push(Buffer.from(c));
  const parsed = JSON.parse(Buffer.concat(chunks).toString("utf8")) as unknown[];
  expect(Array.isArray(parsed)).toBe(true);
  expect(parsed.length).toBeGreaterThan(0);
});
