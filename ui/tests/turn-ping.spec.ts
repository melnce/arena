import { expect, test, type Page } from "@playwright/test";
import { openSettings, waitTransitionEnd } from "./helpers.ts";

const ONE_COST = "10052110";
const SEED = "8484";

async function installFakeAudio(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const starts: number[] = [];
    (window as unknown as { __fakeAudio: { starts: number[] } }).__fakeAudio = { starts };

    class FakeOscillator {
      frequency = { setValueAtTime: () => {} };
      type = "sine";
      connect() {
        return this;
      }
      start() {
        starts.push(performance.now());
      }
      stop() {}
    }

    class FakeGain {
      gain = {
        setValueAtTime: () => {},
        linearRampToValueAtTime: () => {},
        exponentialRampToValueAtTime: () => {},
      };
      connect() {
        return this;
      }
    }

    class FakeAudioContext {
      state: AudioContextState = "suspended";
      destination = {};
      sampleRate = 44_100;
      currentTime = 0;
      createOscillator() {
        return new FakeOscillator();
      }
      createGain() {
        return new FakeGain();
      }
      createBuffer() {
        return { getChannelData: () => new Float32Array(1) };
      }
      createBufferSource() {
        return { connect: () => {}, start: () => {}, buffer: null };
      }
      resume() {
        this.state = "running";
        return Promise.resolve();
      }
    }

    const Ctor = FakeAudioContext as unknown as typeof AudioContext;
    window.AudioContext = Ctor;
    (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext = Ctor;
  });
}

async function fakeOscillatorStarts(page: Page): Promise<number> {
  return page.evaluate(() => (window as unknown as { __fakeAudio: { starts: number[] } }).__fakeAudio.starts.length);
}

async function turnPings(page: Page): Promise<number> {
  return page.evaluate(() => window.__arena!.turnPings!());
}

async function boot(page: Page) {
  await page.goto("/?localbot=0");
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
  await page.locator("#hideBotHandToggle").check();
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

/** After reload, share params auto-start the game and close the drawer — wait before opening settings. */
async function waitRestoredGameAndClosedDrawer(page: Page): Promise<void> {
  const drawer = page.locator("#settingsDrawer");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 30_000,
  });
  await expect(drawer).not.toHaveClass(/open/, { timeout: 30_000 });
  await waitTransitionEnd(drawer, "transform");
}

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

async function waitHumanTurn(page: Page, human: "a" | "b" = "a", timeout = 30_000) {
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", human, { timeout });
}

async function waitBotPlay(page: Page, timeout = 30_000) {
  await expect
    .poll(
      async () => {
        const lines = (await page.locator("#eventLog").innerText()).split("\n");
        for (let i = lines.length - 1; i >= 0; i--) {
          const line = lines[i]?.trim();
          if (!line) continue;
          try {
            const ev = JSON.parse(line) as { play?: { player?: string } };
            if (ev.play?.player === "b") return true;
          } catch {
            /* skip */
          }
        }
        return false;
      },
      { timeout },
    )
    .toBeTruthy();
}

async function setupVsBotHumanFirst(page: Page) {
  const humanDeck = await importDeck(page, "ping-human.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "ping-bot.json", { [ONE_COST]: 40 });
  await startVsBot(page, {
    seed: SEED,
    humanDeck,
    botDeck,
    human: "a",
    first: "a",
    botPolicy: "first-legal",
  });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
}

async function assertPingCounts(page: Page, expected: number) {
  await expect.poll(() => turnPings(page)).toBe(expected);
  await expect.poll(() => fakeOscillatorStarts(page)).toBe(expected * 2);
}

test.describe("vs-bot turn ping", () => {
  test.beforeEach(async ({ page }) => {
    await installFakeAudio(page);
  });

  test("human first: one ping per bot turn end", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    await setupVsBotHumanFirst(page);
    await page.waitForTimeout(800);
    expect(await turnPings(page)).toBe(0);

    await endHumanTurn(page, "a");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 15_000 });
    expect(await turnPings(page)).toBe(0);

    await waitHumanTurn(page, "a");
    await assertPingCounts(page, 1);

    await endHumanTurn(page, "a");
    await waitHumanTurn(page, "a");
    await assertPingCounts(page, 2);
  });

  test("bot first: no ping until bot ends its first turn", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    const humanDeck = await importDeck(page, "ping-human-bf.json", { [ONE_COST]: 40 });
    const botDeck = await importDeck(page, "ping-bot-bf.json", { [ONE_COST]: 40 });
    await startVsBot(page, {
      seed: SEED,
      humanDeck,
      botDeck,
      human: "a",
      first: "b",
      botPolicy: "first-legal",
    });
    await closeDrawer(page);
    expect(await turnPings(page)).toBe(0);
    await confirmHumanMulligan(page, "a");
    expect(await turnPings(page)).toBe(0);
    await waitHumanTurn(page, "a");
    await assertPingCounts(page, 1);
  });

  test("undo and redo do not ping; slow redo resumes bot", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    await setupVsBotHumanFirst(page);
    await endHumanTurn(page, "a");
    await waitHumanTurn(page, "a");
    await assertPingCounts(page, 1);

    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press("Control+z");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
    await expect(page.locator("#redoBtn")).toBeEnabled();
    await page.keyboard.press("Control+y");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
    await assertPingCounts(page, 1);

    await endHumanTurn(page, "a");
    await waitBotPlay(page);
    await page.keyboard.press("Control+z");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a");
    await page.waitForTimeout(600);
    await page.keyboard.press("Control+y");
    await waitHumanTurn(page, "a", 10_000);
    await assertPingCounts(page, 2);
  });

  test("toggle off blocks ping; reload persists; on plays preview", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    await setupVsBotHumanFirst(page);

    await endHumanTurn(page, "a");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", {
      timeout: 15_000,
    });
    await waitHumanTurn(page, "a");
    await assertPingCounts(page, 1);

    await openSettings(page);
    await page.locator("#turnPingToggle").uncheck();
    await closeDrawer(page);

    await endHumanTurn(page, "a");
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", {
      timeout: 15_000,
    });
    await waitHumanTurn(page, "a");
    await expect.poll(() => turnPings(page)).toBe(1);

    await page.reload();
    await waitRestoredGameAndClosedDrawer(page);
    await openSettings(page);
    const drawer = page.locator("#settingsDrawer");
    const toggle = page.locator("#turnPingToggle");
    await expect(toggle).toBeVisible();
    await expect(toggle).not.toBeChecked();
    await drawer.evaluate((el) => {
      el.querySelector("#turnPingToggle")?.scrollIntoView({ block: "center", inline: "nearest" });
    });
    await toggle.scrollIntoViewIfNeeded();
    await toggle.check();
    await assertPingCounts(page, 1);
  });

  test("hotseat and watch produce no pings", async ({ page }) => {
    test.setTimeout(120_000);
    await boot(page);
    const deck = await importDeck(page, "ping-hotseat.json", { [ONE_COST]: 40 });
    await openSettings(page);
    await page.locator("#modeSelect").selectOption("hotseat");
    await page.locator("#seedInput").fill(SEED);
    await page.locator("#blueDeckSelect").selectOption(deck);
    await page.locator("#redDeckSelect").selectOption(deck);
    await page.locator("#policyASelect").selectOption("first-legal", { force: true });
    await page.locator("#policyBSelect").selectOption("first-legal", { force: true });
    await page.locator("#startGameBtn").click();
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
      timeout: 15_000,
    });
    for (let i = 0; i < 2; i++) {
      const btn = page.locator(".mulligan-confirm-btn").locator("visible=true");
      if (await btn.count()) {
        await btn.first().click();
        await expect(btn).toBeHidden({ timeout: 5000 }).catch(() => undefined);
      }
    }
    await closeDrawer(page);
    for (let t = 0; t < 3; t++) {
      const end = page.locator("#endTurnBlue:visible, #endTurnRed:visible").first();
      if (await end.count()) {
        await end.click();
        await page.waitForTimeout(400);
      }
    }
    expect(await turnPings(page)).toBe(0);

    await openSettings(page);
    await page.locator("#modeSelect").selectOption("watch");
    await page.locator("#seedInput").fill("1111");
    await page.locator("#startGameBtn").click();
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
      timeout: 15_000,
    });
    await page.locator("#watchPlayBtn").click();
    await page.waitForTimeout(2000);
    await page.locator("#watchPauseBtn").click();
    expect(await turnPings(page)).toBe(0);
  });

  test("playTurnPing with suspended context does not throw or start oscillators", async ({ page }) => {
    await boot(page);
    const result = await page.evaluate(() => {
      try {
        window.__arena!.playTurnPing!();
        return {
          ok: true,
          starts: (window as unknown as { __fakeAudio: { starts: number[] } }).__fakeAudio.starts.length,
        };
      } catch (err) {
        return { ok: false, error: String(err), starts: -1 };
      }
    });
    expect(result.ok).toBe(true);
    expect(result.starts).toBe(0);
    expect(await turnPings(page)).toBe(1);
  });
});
