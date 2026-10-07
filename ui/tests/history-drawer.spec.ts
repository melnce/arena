import { readFileSync } from "node:fs";
import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings, waitTransitionEnd } from "./helpers.ts";

type PositionLog = {
  v: 1;
  kind: "replay-log";
  seed: string;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  first: string;
  actions: unknown[];
};

type DestroyedRow = { name: string; count: number };

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
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

async function openHistoryDrawer(page: Page) {
  await page.locator("#historyToggle").click();
  await expect(page.locator("#historyDrawer")).toHaveClass(/open/);
  await waitTransitionEnd(page.locator("#historyDrawer"), "transform");
}

function loadFixture(name: string): PositionLog {
  const raw = readFileSync(new URL(`./fixtures/${name}`, import.meta.url), "utf8");
  const m = /"seed"\s*:\s*"(\d+)"/.exec(raw);
  if (!m) throw new Error(`fixture ${name} missing string seed`);
  const log = JSON.parse(raw) as PositionLog;
  expect(log.seed).toBe(m[1]);
  return log;
}

async function loadLog(page: Page, log: PositionLog) {
  await page.evaluate((raw) => window.__arena!.loadLog(raw), log);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main|choice|terminal/, {
    timeout: 30_000,
  });
}

function parseHistRows(labels: string[]): DestroyedRow[] {
  return labels.map((text) => {
    const m = text.match(/^(.+?) ×(\d+)/);
    if (!m) throw new Error(`bad hist label: ${text}`);
    return { name: m[1].trim(), count: Number(m[2]) };
  });
}

async function destroyedRows(page: Page, side: "blue" | "red"): Promise<DestroyedRow[]> {
  const id = side === "blue" ? "#blueDestroyedList" : "#redDestroyedList";
  const labels = await page.locator(`${id} .hist-label`).allTextContents();
  return parseHistRows(labels);
}

async function expectedDestroyed(page: Page, player: "a" | "b"): Promise<DestroyedRow[]> {
  return page.evaluate((who) => {
    const full = window.__arena!.full() as {
      players: { a: { destroyed_history: Array<{ card: string }> }; b: { destroyed_history: Array<{ card: string }> } };
    };
    const cards = full.players[who].destroyed_history.map((r) => r.card);
    const groups = new Map<string, { name: string; cost: number; count: number }>();
    for (const card of cards) {
      const info = window.__arena!.cardText(card) as { name?: string; cost?: number | null };
      const cost = info.cost ?? 0;
      const name = info.name || card;
      const key = `${name}||${cost}`;
      const g = groups.get(key) || { name, cost, count: 0 };
      g.count += 1;
      groups.set(key, g);
    }
    return [...groups.values()]
      .sort((a, b) => a.cost - b.cost || a.name.localeCompare(b.name))
      .map((g) => ({ name: g.name, count: g.count }));
  }, player);
}

async function playedLabels(page: Page, side: "blue" | "red"): Promise<string[]> {
  const id = side === "blue" ? "#bluePlayedList" : "#redPlayedList";
  return page.locator(`${id} .hist-label`).allTextContents();
}

async function endTurnApply(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => "end_turn" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
}

async function applyPlayOrAttack(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => "play" in a) ?? legal.find((a) => "attack" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
}

async function driveUntilBothPlayed(page: Page, maxSteps = 80) {
  for (let i = 0; i < maxSteps; i++) {
    const blue = await playedLabels(page, "blue");
    const red = await playedLabels(page, "red");
    if (blue.length > 0 && red.length > 0) return;
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    if (phase === "choice") {
      await page.evaluate(() => {
        const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
        const act = legal.find((a) => "choose" in a) ?? legal.find((a) => "confirm" in a);
        if (act) window.__arena!.apply(act);
      });
      continue;
    }
    if (await applyPlayOrAttack(page)) continue;
    if (await endTurnApply(page)) continue;
    break;
  }
}

async function driveMain(page: Page, steps: number) {
  for (let i = 0; i < steps; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    if (phase === "choice") {
      await page.evaluate(() => {
        const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
        const act = legal.find((a) => "choose" in a) ?? legal.find((a) => "confirm" in a);
        if (act) window.__arena!.apply(act);
      });
      continue;
    }
    if (await applyPlayOrAttack(page)) continue;
    if (await endTurnApply(page)) continue;
    break;
  }
}

async function assertDestroyedMatchesEngine(page: Page) {
  for (const side of ["blue", "red"] as const) {
    const player = side === "blue" ? "a" : "b";
    const drawer = await destroyedRows(page, side);
    const engine = await expectedDestroyed(page, player);
    expect(drawer).toEqual(engine);
  }
}

test("review11-5935752092966957148 destroyed lists match engine destroyed_history", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const log = loadFixture("review11-5935752092966957148.json");
  await loadLog(page, log);
  await openHistoryDrawer(page);
  await assertDestroyedMatchesEngine(page);

  const obsessed = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { destroyed_history: Array<{ card: string }> } };
    };
    const count = full.players.a.destroyed_history.filter((r) => r.card === "10931110").length;
    const info = window.__arena!.cardText("10931110") as { name?: string };
    const row = [...document.querySelectorAll("#blueDestroyedList .hist-label")]
      .map((el) => el.textContent ?? "")
      .find((t) => t.includes(info.name || "Obsessed Test Subject"));
    const m = row?.match(/×(\d+)/);
    return { engine: count, drawer: m ? Number(m[1]) : 0 };
  });
  expect(obsessed.drawer).toBe(obsessed.engine);
  expect(obsessed.engine).toBeGreaterThan(0);

  await artShot(page.locator("#historyDrawer"), `${ART}/history_drawer_review11.png`);
});

test("review11-10897487595688697553 countdown amulet appears in Blue destroyed", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const log = loadFixture("review11-10897487595688697553.json");
  await loadLog(page, log);
  await openHistoryDrawer(page);
  await assertDestroyedMatchesEngine(page);

  const world = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { destroyed_history: Array<{ card: string }> } };
    };
    const engine = full.players.a.destroyed_history.some((r) => r.card === "10503210");
    const info = window.__arena!.cardText("10503210") as { name?: string };
    const drawer = [...document.querySelectorAll("#blueDestroyedList .hist-label")]
      .map((el) => el.textContent ?? "")
      .some((t) => t.includes(info.name || "World of Games"));
    return { engine, drawer };
  });
  expect(world.engine).toBe(true);
  expect(world.drawer).toBe(true);
});

test("F7 restore keeps played history, ply, and event log prefix", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("42");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  await openHistoryDrawer(page);
  await driveUntilBothPlayed(page);
  const beforeCp = {
    bluePlayed: await playedLabels(page, "blue"),
    redPlayed: await playedLabels(page, "red"),
    ply: await page.locator("#turnCounter").textContent(),
    eventPrefix: await page.locator("#eventLog").textContent(),
    eventCount: await page.locator("#eventLog").getAttribute("data-count"),
  };
  expect(beforeCp.bluePlayed.length).toBeGreaterThan(0);
  expect(beforeCp.redPlayed.length).toBeGreaterThan(0);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("F6");
  await driveMain(page, 5);
  await page.keyboard.press("F7");
  await driveMain(page, 1);

  const after = {
    bluePlayed: await playedLabels(page, "blue"),
    redPlayed: await playedLabels(page, "red"),
    ply: await page.locator("#turnCounter").textContent(),
    eventLog: await page.locator("#eventLog").textContent(),
    eventCount: await page.locator("#eventLog").getAttribute("data-count"),
  };

  for (const label of beforeCp.bluePlayed) {
    expect(after.bluePlayed.some((row) => row.startsWith(label.split(" Set")[0]!))).toBe(true);
  }
  for (const label of beforeCp.redPlayed) {
    expect(after.redPlayed.some((row) => row.startsWith(label.split(" Set")[0]!))).toBe(true);
  }
  expect(Number(after.ply)).toBeGreaterThanOrEqual(Number(beforeCp.ply));
  expect(after.eventLog?.startsWith(beforeCp.eventPrefix ?? "")).toBe(true);
  expect(Number(after.eventCount)).toBeGreaterThanOrEqual(Number(beforeCp.eventCount));
});

test("F8 reroll keeps played history, ply, and event log prefix", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("43");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption("basic-forest");
  await page.locator("#redDeckSelect").selectOption("basic-rune");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  await openHistoryDrawer(page);
  await driveUntilBothPlayed(page);
  const beforeCp = {
    bluePlayed: await playedLabels(page, "blue"),
    redPlayed: await playedLabels(page, "red"),
    ply: await page.locator("#turnCounter").textContent(),
    eventPrefix: await page.locator("#eventLog").textContent(),
    eventCount: await page.locator("#eventLog").getAttribute("data-count"),
  };

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("F6");
  await driveMain(page, 5);
  await page.keyboard.press("F8");
  await driveMain(page, 1);

  const after = {
    bluePlayed: await playedLabels(page, "blue"),
    redPlayed: await playedLabels(page, "red"),
    ply: await page.locator("#turnCounter").textContent(),
    eventLog: await page.locator("#eventLog").textContent(),
    eventCount: await page.locator("#eventLog").getAttribute("data-count"),
  };

  for (const label of beforeCp.bluePlayed) {
    expect(after.bluePlayed.some((row) => row.startsWith(label.split(" Set")[0]!))).toBe(true);
  }
  for (const label of beforeCp.redPlayed) {
    expect(after.redPlayed.some((row) => row.startsWith(label.split(" Set")[0]!))).toBe(true);
  }
  expect(Number(after.ply)).toBeGreaterThanOrEqual(Number(beforeCp.ply));
  expect(after.eventLog?.startsWith(beforeCp.eventPrefix ?? "")).toBe(true);
  expect(Number(after.eventCount)).toBeGreaterThanOrEqual(Number(beforeCp.eventCount));
});

test("undo and redo across a destroy updates destroyed rows", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const log = loadFixture("review11-5935752092966957148.json");
  await loadLog(page, log);
  await openHistoryDrawer(page);
  const snap = { blue: await destroyedRows(page, "blue"), red: await destroyedRows(page, "red") };
  expect(snap.blue.length + snap.red.length).toBeGreaterThan(0);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  let undone = snap;
  for (let i = 0; i < 20; i++) {
    const canUndo = await page.evaluate(() => window.__arena!.canUndo());
    if (!canUndo) break;
    await page.keyboard.press("Control+z");
    undone = { blue: await destroyedRows(page, "blue"), red: await destroyedRows(page, "red") };
    if (JSON.stringify(undone) !== JSON.stringify(snap)) break;
  }
  expect(undone).not.toEqual(snap);

  await page.keyboard.press("Control+y");
  const restored = { blue: await destroyedRows(page, "blue"), red: await destroyedRows(page, "red") };
  expect(restored).toEqual(snap);
});
