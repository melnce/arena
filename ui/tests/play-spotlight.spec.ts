import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const ONE_COST = "10052110";

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
    hideBotHand?: boolean;
    activeOnBottom?: boolean;
  },
) {
  await openSettings(page);
  if (opts.activeOnBottom) {
    await page.locator("#activeOnBottomToggle").check();
  }
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
  const hide = opts.hideBotHand ?? true;
  if (hide) await page.locator("#hideBotHandToggle").check();
  else await page.locator("#hideBotHandToggle").uncheck();
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

async function endHumanTurn(page: Page, human: "a" | "b" = "a") {
  const id = human === "a" ? "endTurnBlue" : "endTurnRed";
  const btn = page.locator(`#${id}:visible`);
  await expect(btn).toHaveCount(1, { timeout: 15_000 });
  await btn.click();
}

async function waitHumanTurn(page: Page, human: "a" | "b" = "a", timeout = 30_000) {
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", human, { timeout });
}

async function waitSpotlightVisible(page: Page, timeout = 20_000) {
  await expect(page.locator("#botPlaySpotlight.visible .spotlight-card")).toBeVisible({ timeout });
}

async function lastBotPlayCard(page: Page): Promise<string | null> {
  return page.evaluate(() => {
    const log = window.__arena!.spotlightLog();
    return log.length ? log[log.length - 1]!.card : null;
  });
}

function rectsOverlap(
  a: { left: number; top: number; right: number; bottom: number },
  b: { left: number; top: number; right: number; bottom: number },
  pad = 2,
): boolean {
  return !(
    a.right + pad < b.left ||
    a.left - pad > b.right ||
    a.bottom + pad < b.top ||
    a.top - pad > b.bottom
  );
}

async function assertSpotlightGeometry(page: Page) {
  const snap = await page.evaluate(() => {
    const spotlight = document.getElementById("botPlaySpotlight");
    const sr = spotlight?.getBoundingClientRect();
    const collect = (sel: string) =>
      Array.from(document.querySelectorAll<HTMLElement>(sel)).map((el) => {
        const r = el.getBoundingClientRect();
        return { left: r.left, top: r.top, right: r.right, bottom: r.bottom, id: el.id || sel };
      });
    const humanSide = window.__arena!.humanSide();
    const humanHand = humanSide === "a" ? "#blueHand .card" : "#redHand .card";
    return {
      pointerEvents: spotlight ? getComputedStyle(spotlight).pointerEvents : "",
      spotlight: sr
        ? { left: sr.left, top: sr.top, right: sr.right, bottom: sr.bottom }
        : null,
      forbidden: [
        ...collect("#redBoard .card"),
        ...collect("#blueBoard .card"),
        ...collect(humanHand),
        ...collect("#redLeader"),
        ...collect("#blueLeader"),
        ...collect("#turnControls"),
      ],
    };
  });
  expect(snap.pointerEvents).toBe("none");
  expect(snap.spotlight).not.toBeNull();
  for (const zone of snap.forbidden) {
    expect(
      rectsOverlap(snap.spotlight!, zone),
      `spotlight overlaps ${zone.id}`,
    ).toBe(false);
  }
}

test("bot play spotlight shows the played card and clears after ~1.5s", async ({ page }) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "4242", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  const card = await page.locator("#botPlaySpotlight .spotlight-card").getAttribute("data-card");
  expect(card).toBeTruthy();
  expect(card).toBe(ONE_COST);
  const playCard = await page.evaluate(() => {
    const lines = document.getElementById("eventLog")?.textContent?.split("\n") ?? [];
    for (let i = lines.length - 1; i >= 0; i--) {
      const line = lines[i]?.trim();
      if (!line) continue;
      const ev = JSON.parse(line) as { play?: { player?: string; card?: string } };
      if (ev.play?.player === "b") return ev.play.card ?? null;
    }
    return null;
  });
  expect(playCard).toBe(ONE_COST);
  expect(card).toBe(playCard);
  const log = await page.evaluate(() => window.__arena!.spotlightLog());
  expect(log.length).toBeGreaterThanOrEqual(1);
  expect(log[0]!.card).toBe(card);
  expect(log[0]!.card).toBe(ONE_COST);
  const img = page.locator("#botPlaySpotlight .spotlight-img");
  const hasFallback = await page.locator("#botPlaySpotlight .card-fallback").count();
  if (hasFallback) {
    await expect(page.locator("#botPlaySpotlight .card-fallback .fb-name")).not.toBeEmpty();
  } else {
    const src = await img.getAttribute("src");
    const expected = await img.getAttribute("data-expected-src");
    expect(src).toBeTruthy();
    if (expected) expect(src).toBe(expected);
  }
  await artShot(page.locator("#appRoot"), `${ART}/spotlight_bot_top_1440.png`);
  const shownAt = log[0]!.shownAt;
  await expect
    .poll(async () => (await page.evaluate(() => window.__arena!.spotlightLog()[0]?.hiddenAt ?? 0)) > 0)
    .toBeTruthy();
  const after = await page.evaluate(() => window.__arena!.spotlightLog());
  expect(after[0]!.hiddenAt - shownAt).toBeGreaterThanOrEqual(1150);
  expect(after[0]!.hiddenAt - shownAt).toBeLessThanOrEqual(2200);
  await expect(page.locator("#botPlaySpotlight.visible")).toHaveCount(0);
});

test("human plays do not show the spotlight", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human2.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot2.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "4242", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  const playable = page.locator("#blueHand .card.playable-glow, #blueHand .card.legal-play");
  await expect(playable.first()).toBeVisible({ timeout: 10_000 });
  await playable.first().click({ button: "right" });
  await expect(page.locator("#botPlaySpotlight.visible")).toHaveCount(0);
  const log = await page.evaluate(() => window.__arena!.spotlightLog());
  expect(log).toEqual([]);
});

test("hidden hand still spotlights only the played card", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human3.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot3.json", { [ONE_COST]: 40 });
  await startVsBot(page, {
    seed: "6262",
    humanDeck,
    botDeck,
    botPolicy: "first-legal",
    hideBotHand: true,
  });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  const snap = await page.evaluate(() => {
    const overlay = document.getElementById("botPlaySpotlight");
    const ids = overlay
      ? Array.from(overlay.querySelectorAll<HTMLElement>("[data-card]")).map((el) => el.dataset.card)
      : [];
    const hand = Array.from(
      document.querySelectorAll<HTMLElement>("#redHand .card[data-card]"),
    ).map((el) => el.dataset.card);
    return { ids, hand };
  });
  expect(snap.ids.length).toBe(1);
  expect(snap.hand.every((id) => !id)).toBe(true);
});

test("two bot plays in one turn respect minimum visible time", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human4.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot4.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "7373", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitHumanTurn(page, "a");
  const before = await page.evaluate(() => window.__arena!.spotlightLog().length);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  await expect
    .poll(async () => {
      const log = await page.evaluate(() => window.__arena!.spotlightLog());
      if (log.length < before + 2) return false;
      return log.slice(before, before + 2).every((e) => e.hiddenAt > 0);
    }, { timeout: 30_000 })
    .toBeTruthy();
  const log = await page.evaluate(() => window.__arena!.spotlightLog());
  const entries = log.slice(before, before + 2);
  expect(entries.length).toBe(2);
  for (const entry of entries) {
    expect(entry.hiddenAt - entry.shownAt).toBeGreaterThanOrEqual(1150);
  }
});

test("undo clears the spotlight immediately; redo does not replay it", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human5.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot5.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "8484", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("Control+z");
  const cleared = await page.evaluate(() => ({
    visible: document.querySelectorAll("#botPlaySpotlight.visible").length,
    cards: document.querySelectorAll("#botPlaySpotlight .spotlight-card").length,
  }));
  expect(cleared.visible).toBe(0);
  expect(cleared.cards).toBe(0);
  await page.waitForTimeout(600);
  const beforeRedo = await page.evaluate(() => window.__arena!.spotlightLog().length);
  await page.keyboard.press("Control+y");
  await page.waitForTimeout(1500);
  const afterRedo = await page.evaluate(() => ({
    len: window.__arena!.spotlightLog().length,
    visible: document.querySelectorAll("#botPlaySpotlight.visible").length,
  }));
  expect(afterRedo.len).toBe(beforeRedo);
  expect(afterRedo.visible).toBe(0);
});

test("fast undo then redo: live bot plays after redo each get one spotlight", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human5b.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot5b.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "8484", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  await page.keyboard.press("Control+z");
  await page.keyboard.press("Control+y");
  const snap = await page.evaluate(() => {
    const L0 = window.__arena!.spotlightLog().length;
    const lines = document.getElementById("eventLog")?.textContent?.split("\n") ?? [];
    let P0 = 0;
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      try {
        const ev = JSON.parse(trimmed) as { play?: { player?: string } };
        if (ev.play?.player === "b") P0 += 1;
      } catch {
        /* skip malformed lines */
      }
    }
    return { L0, P0 };
  });
  await page.waitForTimeout(2500);
  const after = await page.evaluate(() => {
    const logLen = window.__arena!.spotlightLog().length;
    const lines = document.getElementById("eventLog")?.textContent?.split("\n") ?? [];
    let bPlays = 0;
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      try {
        const ev = JSON.parse(trimmed) as { play?: { player?: string } };
        if (ev.play?.player === "b") bPlays += 1;
      } catch {
        /* skip malformed lines */
      }
    }
    return { logLen, bPlays };
  });
  expect(after.logLen - snap.L0).toBe(after.bPlays - snap.P0);
});

test("spotlight stays behind the open settings drawer", async ({ page }) => {
  test.setTimeout(90_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human-drawer.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot-drawer.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "4242", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  await openSettings(page);
  const behindDrawer = await page.evaluate(() => {
    const overlay = document.getElementById("botPlaySpotlight");
    if (!overlay) return false;
    const r = overlay.getBoundingClientRect();
    const x = r.left + r.width / 2;
    const y = r.top + r.height / 2;
    overlay.style.pointerEvents = "auto";
    const hit = document.elementFromPoint(x, y);
    overlay.style.pointerEvents = "none";
    return hit?.closest("#settingsDrawer") != null;
  });
  expect(behindDrawer).toBe(true);
});

const CARD_Z = "10001110";

test("clear during fade-out does not corrupt the next spotlight queue", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human-drain.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot-drain.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "5252", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await page.evaluate(() => {
    window.__arena!.spotlightEnqueue([{ play: { player: "b", card: "10052110", form: "normal" } }]);
  });
  await waitSpotlightVisible(page);
  await page.evaluate(
    async ({ y, z }) => {
      await new Promise<void>((resolve) => {
        function waitHiding() {
          const overlay = document.getElementById("botPlaySpotlight");
          if (!overlay?.classList.contains("hiding")) {
            requestAnimationFrame(waitHiding);
            return;
          }
          const toggle = document.getElementById("botPlaySpotlightToggle") as HTMLInputElement;
          toggle.checked = false;
          toggle.dispatchEvent(new Event("change", { bubbles: true }));
          toggle.checked = true;
          toggle.dispatchEvent(new Event("change", { bubbles: true }));
          window.__arena!.spotlightEnqueue([
            { play: { player: "b", card: y, form: "normal" } },
            { play: { player: "b", card: z, form: { enhance: 2 } } },
          ]);
          resolve();
        }
        requestAnimationFrame(waitHiding);
      });
    },
    { y: ONE_COST, z: CARD_Z },
  );
  const cardCounts = await page.evaluate(async () => {
    const counts: number[] = [];
    const start = performance.now();
    return new Promise<number[]>((resolve) => {
      function sample() {
        const overlay = document.getElementById("botPlaySpotlight");
        if (overlay?.classList.contains("visible")) {
          counts.push(overlay.querySelectorAll(".spotlight-card").length);
        }
        if (performance.now() - start < 3500) requestAnimationFrame(sample);
        else resolve(counts);
      }
      requestAnimationFrame(sample);
    });
  });
  for (const n of cardCounts) expect(n).toBe(1);
  const log = await page.evaluate(() => window.__arena!.spotlightLog());
  expect(log.length).toBeGreaterThanOrEqual(3);
  const yEntry = log[log.length - 2]!;
  const zEntry = log[log.length - 1]!;
  expect(yEntry.card).toBe(ONE_COST);
  expect(zEntry.card).toBe(CARD_Z);
  expect(yEntry.hiddenAt - yEntry.shownAt).toBeGreaterThanOrEqual(1150);
  expect(zEntry.hiddenAt - zEntry.shownAt).toBeGreaterThanOrEqual(1150);
  expect(zEntry.shownAt).toBeGreaterThanOrEqual(yEntry.shownAt);
});

test("setting off disables spotlight and survives reload", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  await openSettings(page);
  await page.locator("#botPlaySpotlightToggle").uncheck();
  const humanDeck = await importDeck(page, "spot-human6.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot6.json", { [ONE_COST]: 40 });
  await startVsBot(page, { seed: "9595", humanDeck, botDeck, botPolicy: "first-legal" });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitHumanTurn(page, "a", 25_000);
  await expect(page.locator("#botPlaySpotlight.visible")).toHaveCount(0);
  expect(await page.evaluate(() => window.__arena!.spotlightLog())).toEqual([]);
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await expect(page.locator("#botPlaySpotlightToggle")).not.toBeChecked();
});

test("watch mode never shows the spotlight", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const deck = await importDeck(page, "spot-watch.json", { [ONE_COST]: 40 });
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("watch");
  await page.locator("#seedInput").fill("1111");
  await page.locator("#blueDeckSelect").selectOption(deck);
  await page.locator("#redDeckSelect").selectOption(deck);
  await page.locator("#policyASelect").selectOption("first-legal", { force: true });
  await page.locator("#policyBSelect").selectOption("first-legal", { force: true });
  await page.locator("#startGameBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await page.locator("#watchSpeed").evaluate((el) => {
    (el as HTMLInputElement).value = "20";
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await page.locator("#watchPlayBtn").click();
  await page.waitForTimeout(2500);
  await expect(page.locator("#botPlaySpotlight.visible")).toHaveCount(0);
  expect(await page.evaluate(() => window.__arena!.spotlightLog())).toEqual([]);
});

test("spotlight geometry avoids boards, human hand, leaders, and rail", async ({ page }) => {
  test.setTimeout(180_000);
  const viewports = [
    { width: 1440, height: 900, shot: "spotlight_bot_top_1440.png" },
    { width: 1180, height: 820, shot: "spotlight_bot_top_1180.png" },
  ] as const;
  for (const vp of viewports) {
    await page.setViewportSize({ width: vp.width, height: vp.height });
    await boot(page);
    const humanDeck = await importDeck(page, `spot-human-geo-${vp.width}.json`, { [ONE_COST]: 40 });
    const botDeck = await importDeck(page, `spot-bot-geo-${vp.width}.json`, { [ONE_COST]: 40 });
    await startVsBot(page, { seed: "13579", humanDeck, botDeck, botPolicy: "first-legal" });
    await confirmHumanMulligan(page, "a");
    await closeDrawer(page);
    for (let i = 0; i < 4; i++) {
      await waitHumanTurn(page, "a", 25_000);
      await endHumanTurn(page, "a");
      await page.waitForTimeout(1200);
    }
    const botUnits = await page.evaluate(
      () => document.querySelectorAll("#redBoard .card").length,
    );
    expect(botUnits).toBeGreaterThanOrEqual(3);
    await waitSpotlightVisible(page);
    await assertSpotlightGeometry(page);
    await artShot(page.locator("#appRoot"), `${ART}/${vp.shot}`);
  }
});

test("active on bottom spotlight screenshot during bot turn", async ({ page }) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  await boot(page);
  const humanDeck = await importDeck(page, "spot-human-bottom.json", { [ONE_COST]: 40 });
  const botDeck = await importDeck(page, "spot-bot-bottom.json", { [ONE_COST]: 40 });
  await startVsBot(page, {
    seed: "24680",
    humanDeck,
    botDeck,
    botPolicy: "first-legal",
    activeOnBottom: true,
  });
  await confirmHumanMulligan(page, "a");
  await closeDrawer(page);
  await endHumanTurn(page, "a");
  await waitSpotlightVisible(page);
  await artShot(page.locator("#appRoot"), `${ART}/spotlight_active_on_bottom.png`);
});
