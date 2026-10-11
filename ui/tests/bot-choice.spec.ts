import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const STRONG = "h0:nodes=6000";
const THINK_QUERY = "localbot=0&botworkerspin=2000";

function readPcPolicies(): { strong: string; cheat: string } {
  const servePath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../py/serve.py");
  const text = fs.readFileSync(servePath, "utf8");
  const strongM = text.match(/^DEFAULT_STRONG = "(.*)"$/m);
  const cheatM = text.match(/^DEFAULT_CHEAT = "(.*)"$/m);
  if (!strongM || !cheatM) throw new Error("serve.py policies missing");
  return { strong: strongM[1], cheat: cheatM[1] };
}

const PC = readPcPolicies();

async function boot(page: Page, query = "") {
  await page.goto(`/${query ? `?${query}` : ""}`);
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function importTinyDeck(page: Page) {
  await openSettings(page);
  await page.locator("#deckImportFileInput").setInputFiles({
    name: "tiny.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify({ "10001110": 8 })),
  });
  const id = "import-tiny";
  await expect(page.locator("#blueDeckSelect")).toHaveValue(id, { timeout: 10_000 });
  return id;
}

async function startVsBot(
  page: Page,
  opts: {
    seed?: string;
    first?: string;
    deckA?: string;
    deckB?: string;
    botPolicy?: string;
  },
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill(opts.seed ?? "1");
  await page.locator("#firstSelect").selectOption(opts.first ?? "a");
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
  await page.locator("#humanSideSelect").selectOption("a");
  const policy = opts.botPolicy ?? STRONG;
  await page.locator("#vsBotPolicy").selectOption(policy);
  await page.locator("#policyBSelect").selectOption(policy, { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
}

async function confirmMulligans(page: Page) {
  for (let i = 0; i < 2; i++) {
    const btn = page.locator(".mulligan-confirm-btn:visible");
    if ((await btn.count()) === 0) break;
    await btn.first().click();
    await expect(btn.first()).toBeHidden({ timeout: 5000 }).catch(() => undefined);
  }
}

async function playToGameOver(page: Page) {
  for (let i = 0; i < 40; i++) {
    const phase = await page.locator("#turnCounter").getAttribute("data-phase");
    if (phase === "terminal") break;
    const ended = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
      const act = legal.find((a) => "end_turn" in a);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    });
    if (!ended) break;
  }
  await expect(page.locator("#gameOverOverlay")).toBeVisible({ timeout: 10_000 });
}

async function optionMeta(page: Page, value: string) {
  return page.locator(`#vsBotPolicy option[value="${value}"]`).evaluate((el) => {
    const o = el as HTMLOptionElement;
    return { label: o.textContent ?? "", title: o.title };
  });
}

test.describe("rematch uses menu bot policy", () => {
  for (const rematchBtn of ["#rematchNewSeedBtn", "#rematchSameSeedBtn"] as const) {
    test(`${rematchBtn} picks vsBotPolicy after game on h0 (strong)`, async ({ page }) => {
      test.setTimeout(90_000);
      await boot(page);
      const tiny = await importTinyDeck(page);
      await startVsBot(page, { deckA: tiny, deckB: tiny, botPolicy: STRONG });
      await confirmMulligans(page);
      await playToGameOver(page);
      await openSettings(page);
      await page.locator("#vsBotPolicy").selectOption(PC.strong);
      await page.locator(rematchBtn).click();
      await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan", {
        timeout: 10_000,
      });
      const policies = await page.evaluate(() => window.__arena!.policies());
      expect(policies).toEqual([PC.strong, PC.strong]);
    });
  }
});

test("vsBotPolicyHint shows label and title for every option; persists after reload", async ({
  page,
}) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  const values = await page.locator("#vsBotPolicy option").evaluateAll((opts) =>
    (opts as HTMLOptionElement[]).map((o) => o.value),
  );
  for (const value of values) {
    await page.locator("#vsBotPolicy").selectOption(value);
    const meta = await optionMeta(page, value);
    const hint = page.locator("#vsBotPolicyHint");
    await expect(hint).toContainText(meta.label);
    if (meta.title) await expect(hint).toContainText(meta.title);
  }
  await page.locator("#vsBotPolicy").selectOption("h0");
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  const meta = await optionMeta(page, "h0");
  const hint = page.locator("#vsBotPolicyHint");
  await expect(hint).toContainText(meta.label);
  await expect(hint).toContainText(meta.title);
});

test("menu change mid-game: hint warns; badge names running bot until Start", async ({ page }) => {
  test.setTimeout(60_000);
  await boot(page);
  await startVsBot(page, { botPolicy: STRONG });
  await confirmMulligans(page);
  const strongMeta = await optionMeta(page, STRONG);
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser · ${strongMeta.label}`);
  await openSettings(page);
  const pcMeta = await optionMeta(page, PC.strong);
  await page.locator("#vsBotPolicy").selectOption(PC.strong);
  const hint = page.locator("#vsBotPolicyHint");
  await expect(hint).toContainText(pcMeta.label);
  await expect(hint).toContainText(pcMeta.title);
  await expect(hint).toContainText("Applies from the next game (Start or Rematch).");
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser · ${strongMeta.label}`);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser · ${pcMeta.label}`);
});

test("browser badge names running bot idle and while thinking", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page, THINK_QUERY);
  await startVsBot(page, { seed: "42", first: "b", botPolicy: "h0" });
  await confirmMulligans(page);
  const meta = await optionMeta(page, "h0");
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser · ${meta.label}`);
  await expect(page.locator("#botBackendBadge")).toContainText("thinking", { timeout: 30_000 });
  await expect(page.locator("#botBackendBadge")).toContainText(meta.label);
});

test("watch rematch takes policyASelect and policyBSelect", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const tiny = await importTinyDeck(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("watch");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(tiny);
  await page.locator("#redDeckSelect").selectOption(tiny);
  await page.locator("#policyASelect").selectOption("h0");
  await page.locator("#policyBSelect").selectOption("random");
  await page.locator("#watchSpeed").evaluate((el) => {
    (el as HTMLInputElement).value = "20";
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await playToGameOver(page);
  await openSettings(page);
  await page.locator("#policyASelect").selectOption("first-legal");
  await page.locator("#policyBSelect").selectOption(STRONG);
  await page.locator("#rematchSameSeedBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan", {
    timeout: 10_000,
  });
  const policies = await page.evaluate(() => window.__arena!.policies());
  expect(policies).toEqual(["first-legal", STRONG]);
});

test("tablet screenshot: PC strength hint and badge", async ({ page }) => {
  await page.setViewportSize({ width: 1180, height: 820 });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").selectOption(PC.strong);
  const meta = await optionMeta(page, PC.strong);
  await expect(page.locator("#vsBotPolicyHint")).toContainText(meta.label);
  await artShot(page.locator("#vsBotFields"), `${ART}/bot_choice_pc_hint.png`);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).toContainText("h0 (PC strength)");
  await artShot(page.locator("#appRoot"), `${ART}/bot_choice_pc_badge.png`);
});
