import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import {
  CUSTOM_SPEC_MENU_VALUE,
  fullStrengthPolicyLabel,
  policyOptionLabel,
} from "../src/policy-labels.ts";
import { ART, artShot, openSettings } from "./helpers.ts";

const STRONG = "h0:nodes=6000";
const THINK_QUERY = "localbot=0&botworkerspin=8000";
const CUSTOM_MENU = CUSTOM_SPEC_MENU_VALUE;
const CUSTOM_SPEC = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";

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
  for (let i = 0; i < 4; i++) {
    const clicked = await page.evaluate(() => {
      const btns = [...document.querySelectorAll<HTMLButtonElement>(".mulligan-confirm-btn")];
      const vis = btns.find((b) => b.getClientRects().length > 0);
      if (!vis) return false;
      vis.click();
      return true;
    });
    if (!clicked) break;
    await page.waitForTimeout(150);
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
    if (value === CUSTOM_MENU) continue;
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
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${STRONG})`);
  await openSettings(page);
  const pcMeta = await optionMeta(page, PC.strong);
  await page.locator("#vsBotPolicy").selectOption(PC.strong);
  const hint = page.locator("#vsBotPolicyHint");
  await expect(hint).toContainText(pcMeta.label);
  await expect(hint).toContainText(pcMeta.title);
  await expect(hint).toContainText("Applies from the next game (Start or Rematch).");
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${STRONG})`);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${PC.strong})`);
});

test("browser badge names running bot idle and while thinking", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page, THINK_QUERY);
  await startVsBot(page, { seed: "42", first: "b", botPolicy: "h0" });
  await expect(page.locator("#botBackendBadge")).toHaveText("bot: browser (h0)");
  await confirmMulligans(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 60_000 });
  await expect(page.locator("#botBackendBadge")).toHaveText("bot: browser (h0) — thinking…", {
    timeout: 90_000,
  });
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
  const play = page.locator("#watchPlayBtn");
  if (await play.isVisible()) await play.click();
  await expect(page.locator("#gameOverOverlay")).toBeVisible({ timeout: 120_000 });
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

async function openCustomSpecEditor(page: Page) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").selectOption(CUSTOM_MENU);
  await expect(page.locator("#customBotSpecPanel")).toBeVisible();
}

async function saveCustomSpec(page: Page, spec: string) {
  await page.locator("#customBotSpecInput").fill(spec);
  await page.locator("#customBotSpecSave").click();
}

test("vsBotPolicy labels match the policy table", async ({ page }) => {
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  const opts = await page.locator("#vsBotPolicy option").evaluateAll((els) =>
    (els as HTMLOptionElement[]).map((o) => ({ value: o.value, label: o.textContent ?? "" })),
  );
  const byValue = new Map(opts.map((o) => [o.value, o.label]));
  expect(byValue.get("random")).toBe(policyOptionLabel("random", PC.strong, PC.cheat));
  expect(byValue.get("first-legal")).toBe(policyOptionLabel("first-legal", PC.strong, PC.cheat));
  expect(byValue.get("h0")).toBe(policyOptionLabel("h0", PC.strong, PC.cheat));
  expect(byValue.get(STRONG)).toBe(policyOptionLabel(STRONG, PC.strong, PC.cheat));
  expect(byValue.get("h0:nodes=6000,info=all")).toBe(
    policyOptionLabel("h0:nodes=6000,info=all", PC.strong, PC.cheat),
  );
  expect(byValue.get(PC.strong)).toBe(policyOptionLabel(PC.strong, PC.strong, PC.cheat));
  expect(byValue.get(PC.cheat)).toBe(policyOptionLabel(PC.cheat, PC.strong, PC.cheat));
  expect(byValue.get(CUSTOM_MENU)).toBe("Custom spec…");
});

test("full strength label follows nodes= in the spec string", () => {
  const spec = "h0:nodes=48000,horizon=2";
  expect(fullStrengthPolicyLabel(spec, false)).toBe("Full strength — 48 000 nodes");
  expect(fullStrengthPolicyLabel("h0:horizon=2", false)).toBe("Full strength");
});

test("local server override hint when mocked server is up", async ({ page }) => {
  await page.route("http://127.0.0.1:8765/**", async (route) => {
    if (route.request().url().includes("/health")) {
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
    await route.fulfill({ status: 404, body: "not found" });
  });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").selectOption(STRONG);
  const hint = page.locator("#vsBotPolicyHint");
  await expect(hint).toContainText("Your local bot server is running", { timeout: 10_000 });
  await expect(hint).toContainText("h0:nodes=16000");
  await page.locator("#localBotToggle").uncheck();
  await expect(hint).not.toContainText("Your local bot server is running", { timeout: 5_000 });
});

test("custom spec: save, reload, start and rematch use spec; badge shows spec", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  await openCustomSpecEditor(page);
  await saveCustomSpec(page, CUSTOM_SPEC);
  await expect(page.locator("#vsBotPolicy")).toHaveValue(CUSTOM_SPEC);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#vsBotPolicy")).toHaveValue(CUSTOM_SPEC);
  const tiny = await importTinyDeck(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#seedInput").fill("1");
  await page.locator("#blueDeckSelect").selectOption(tiny);
  await page.locator("#redDeckSelect").selectOption(tiny);
  await page.locator("#humanSideSelect").selectOption("a");
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${CUSTOM_SPEC})`);
  expect(await page.evaluate(() => window.__arena!.policies())).toEqual([CUSTOM_SPEC, CUSTOM_SPEC]);
  await playToGameOver(page);
  await page.locator("#rematchNewSeedBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "mulligan", {
    timeout: 10_000,
  });
  expect(await page.evaluate(() => window.__arena!.policies())).toEqual([CUSTOM_SPEC, CUSTOM_SPEC]);
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${CUSTOM_SPEC})`);
});

test("custom spec: invalid specs rejected and not stored", async ({ page }) => {
  await boot(page);
  for (const bad of ["h0:nodes=abc", "minimax"]) {
    await openCustomSpecEditor(page);
    await saveCustomSpec(page, bad);
    await expect(page.locator("#customBotSpecError")).not.toHaveText("");
    await expect(page.locator("#customBotSpecPanel")).toBeVisible();
    const stored = await page.evaluate(() => localStorage.getItem("svwb.customBotSpec"));
    expect(stored).toBeNull();
  }
});

test("tablet screenshots: menu, custom error, badge with spec", async ({ page }) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await boot(page);
  await openCustomSpecEditor(page);
  await saveCustomSpec(page, CUSTOM_SPEC);
  await artShot(page.locator("#vsBotFields"), `${ART}/bot_choice_menu_custom.png`);
  await page.locator("#vsBotPolicy").selectOption(CUSTOM_MENU);
  await page.locator("#customBotSpecInput").fill("minimax");
  await page.locator("#customBotSpecSave").click();
  await expect(page.locator("#customBotSpecError")).not.toHaveText("");
  await artShot(page.locator("#vsBotFields"), `${ART}/bot_choice_custom_error.png`);
  await page.locator("#customBotSpecCancel").click();
  await page.locator("#vsBotPolicy").selectOption(CUSTOM_SPEC);
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await confirmMulligans(page);
  await expect(page.locator("#botBackendBadge")).toHaveText(`bot: browser (${CUSTOM_SPEC})`);
  await artShot(page.locator("#appRoot"), `${ART}/bot_choice_spec_badge.png`);
});

test("tablet screenshot: policy menu labels", async ({ page }) => {
  await page.setViewportSize({ width: 1180, height: 820 });
  await boot(page);
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("vs-bot");
  await page.locator("#vsBotPolicy").evaluate((el) => {
    (el as HTMLSelectElement).size = 12;
  });
  await artShot(page.locator("#vsBotFields"), `${ART}/bot_choice_menu_labels.png`);
});
