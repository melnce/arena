import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";

type KillDoneFixture = {
  v: number;
  kind: string;
  seed: string;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  deckAId: string;
  deckBId: string;
  first: string;
  actions: unknown[];
  botSeedOffset: number;
  expectedAction: Record<string, unknown>;
};

function readServeStrong(): string {
  const servePath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../py/serve.py");
  const text = fs.readFileSync(servePath, "utf8");
  const strongM = text.match(/^DEFAULT_STRONG = "(.*)"$/m);
  if (!strongM) throw new Error("missing DEFAULT_STRONG in py/serve.py");
  return strongM[1];
}

const STRONG = readServeStrong();

function loadFixture(name: string): KillDoneFixture {
  const p = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);
  return JSON.parse(fs.readFileSync(p, "utf8")) as KillDoneFixture;
}

async function boot(page: Page, query = "") {
  await page.goto(`/${query ? `?${query}` : ""}`);
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function actionLog(page: Page): Promise<unknown[]> {
  return page.evaluate(() => window.__arena!.actions());
}

function toReplayLog(fixture: KillDoneFixture): Record<string, unknown> {
  const { botSeedOffset: _o, expectedAction: _e, ...log } = fixture;
  return log;
}

async function longestReplayablePrefix(page: Page, fixture: KillDoneFixture): Promise<KillDoneFixture> {
  const base = toReplayLog(fixture);
  const actions = fixture.actions as unknown[];
  let replayable = { ...fixture, actions: [] as unknown[] };
  await boot(page, "localbot=0");
  for (let ply = 1; ply <= actions.length; ply++) {
    const candidate = { ...base, actions: actions.slice(0, ply) };
    const ok = await page.evaluate((log) => window.__arena!.tryReplayLog(log), candidate);
    if (!ok.ok) break;
    replayable = { ...fixture, actions: actions.slice(0, ply) };
  }
  expect(replayable.actions.length).toBe(fixture.actions.length);
  return replayable;
}

async function nativeStrongDecision(
  page: Page,
  replayable: KillDoneFixture,
): Promise<{ action: unknown; value: number | null }> {
  const botSeed = String(BigInt(replayable.seed) + BigInt(replayable.botSeedOffset));
  return page.evaluate(
    ({ log, policy, botSeed: seed }) => {
      window.__arena!.loadLog(log);
      const step = JSON.parse(window.__arena!.botAction(policy, seed)) as Record<string, unknown>;
      const valued = JSON.parse(window.__arena!.botActionValue(policy, seed)) as {
        action: unknown;
        value: number | null;
      };
      return { step, value: valued.value };
    },
    { log: toReplayLog(replayable), policy: STRONG, botSeed },
  );
}

async function botMoveAt(
  page: Page,
  replayable: KillDoneFixture,
  query: string,
): Promise<{ move: unknown; meta: { backend: string; error?: string } | undefined }> {
  const prefixLen = replayable.actions.length;
  const acting = await page.evaluate((log) => {
    window.__arena!.loadLog(log);
    return document.getElementById("turnCounter")?.getAttribute("data-acting");
  }, toReplayLog(replayable));
  const humanSide = acting === "a" ? "b" : "a";

  await boot(page, query);
  await page.evaluate((log) => window.__arena!.loadLog(log), toReplayLog(replayable));
  await page.evaluate(
    ({ pol, human }) => window.__arena!.armVsBot({ humanSide: human, botPolicy: pol }),
    { pol: STRONG, human: humanSide },
  );
  await expect
    .poll(async () => (await actionLog(page)).length, { timeout: 600_000 })
    .toBeGreaterThan(prefixLen);
  const log = await actionLog(page);
  const meta = await page.evaluate((ply) => {
    const entries = window.__arena!.captureBotMeta();
    return entries.find((m) => m.ply === ply);
  }, prefixLen);
  return { move: log[prefixLen], meta };
}

const FIXTURES: Array<{ file: string; label: string }> = [
  { file: "pool-kill-attack-leader.json", label: "attack leader kill" },
  { file: "pool-kill-bonus-pp.json", label: "bonus pp kill" },
];

for (const { file, label } of FIXTURES) {
  test.describe(label, () => {
    test.setTimeout(720_000);

    test("native DEFAULT_STRONG agrees with fixture", async ({ page }) => {
      const fixture = loadFixture(file);
      const replayable = await longestReplayablePrefix(page, fixture);
      const native = await nativeStrongDecision(page, replayable);
      expect(native.step).toEqual(fixture.expectedAction);
      expect(native.value).toBe(80);
    });

    test("pool (4 cores) decides without worker error", async ({ page }) => {
      const fixture = loadFixture(file);
      const replayable = await longestReplayablePrefix(page, fixture);
      const { move, meta } = await botMoveAt(page, replayable, "localbot=0&botcores=4");
      expect(move).toEqual(fixture.expectedAction);
      expect(meta).toBeTruthy();
      expect(meta!.backend).toBe("worker");
      expect(meta!.error).toBeUndefined();
      await expect(page.locator("#botBackendBadge")).not.toContainText(/worker error:/i);
    });

    test("single worker (1 core) decides the same action", async ({ page }) => {
      const fixture = loadFixture(file);
      const replayable = await longestReplayablePrefix(page, fixture);
      const { move } = await botMoveAt(page, replayable, "localbot=0&botworker=0");
      expect(move).toEqual(fixture.expectedAction);
    });
  });
}
