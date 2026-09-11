import { expect, test, type Page } from "@playwright/test";
import { mkdir } from "node:fs/promises";

const ART = "/opt/cursor/artifacts";

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

async function startGame(
  page: Page,
  opts: {
    mode?: string;
    seed?: string;
    first?: string;
    deckA?: string;
    deckB?: string;
  } = {},
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode ?? "hotseat");
  if (opts.seed) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function applyAction(page: Page, action: unknown) {
  await page.evaluate((act) => window.__arena!.apply(act), action);
}

async function applyFirst(page: Page, key: string) {
  const ok = await page.evaluate((k) => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => k in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, key);
  expect(ok, `expected legal ${key}`).toBeTruthy();
}

async function playCard(page: Page, card: string) {
  const ok = await page.evaluate((id) => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === id);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  }, card);
  expect(ok, `expected play ${card}`).toBeTruthy();
}

async function endTurnApply(page: Page) {
  await applyFirst(page, "end_turn");
}

async function skipToPp(page: Page, pp: number) {
  for (let i = 0; i < 16; i++) {
    const cur = await page.evaluate(() => {
      const full = window.__arena!.full() as {
        players: { a: { pp: number }; b: { pp: number } };
        active: string;
      };
      return full.players[full.active as "a" | "b"].pp;
    });
    if (cur >= pp) return;
    await endTurnApply(page);
  }
}

function idleGlow(side: "blue" | "red") {
  return [
    `#${side}Hand .legal-play`,
    `#${side}Hand .can-attack`,
    `#${side}Hand .playable-glow`,
    `#${side}Hand .enhance-ready`,
    `#${side}Hand .rush-glow`,
    `#${side}Board .legal-play`,
    `#${side}Board .can-attack`,
    `#${side}Board .playable-glow`,
    `#${side}Board .enhance-ready`,
    `#${side}Board .rush-glow`,
  ].join(", ");
}

test("A1 non-acting side has no glow (first A and first B)", async ({ page }) => {
  await boot(page);
  for (const first of ["a", "b"] as const) {
    await startGame(page, {
      seed: "1",
      first,
      deckA: "basic-forest",
      deckB: "basic-rune",
    });
    await confirmMulligans(page);
    await closeDrawer(page);
    const acting = await page.locator("#turnCounter").getAttribute("data-acting");
    const idle = acting === "a" ? "red" : "blue";
    await expect(page.locator(idleGlow(idle))).toHaveCount(0);
    await mkdir(ART, { recursive: true }).catch(() => undefined);
    await page
      .locator("#appRoot")
      .screenshot({ path: `${ART}/a1_no_glow_first_${first}.png` })
      .catch(() => undefined);
  }
});

test("A2 countdown badge updates and evolved art swaps", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "whirl.json", { "10922110": 40 });
  await page.locator("#redDeckSelect").selectOption(id);
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await skipToPp(page, 3);
  await playCard(page, "10922110");
  const flag = page.locator("#blueBoard .card[data-card='90021210']").first();
  await expect(flag).toBeVisible();
  const before = await flag.locator(".countdown-badge").innerText();
  expect(Number(before)).toBeGreaterThan(0);
  await mkdir(ART, { recursive: true });
  await flag.screenshot({ path: `${ART}/a2_countdown_before.png` });
  await endTurnApply(page);
  await endTurnApply(page);
  const goblet = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    return legal.some((a) => a.play?.card === "90021320");
  });
  if (goblet) {
    await playCard(page, "90021320");
    await expect(flag.locator(".countdown-badge")).not.toHaveText(before);
    await flag.screenshot({ path: `${ART}/a2_countdown_after.png` });
  }

  const evoActs = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { slot: number; super: boolean } }>;
    return legal.filter((a) => a.evolve && !a.evolve.super);
  });
  if (evoActs[0]?.evolve) {
    const slot = evoActs[0].evolve.slot;
    const card = page.locator(`#blueBoard .card[data-slot='${slot}']`);
    const beforeSrc = await card.locator("img").getAttribute("src");
    await applyAction(page, evoActs[0]);
    const afterSrc = await card.locator("img").getAttribute("src");
    expect(afterSrc).toBeTruthy();
    expect(afterSrc).not.toBe(beforeSrc);
    await expect(card.locator(".card-stats.bottom-left")).toHaveClass(/stat-buffed/);
    await expect(card.locator(".card-stats.bottom-right")).toHaveClass(/stat-buffed/);
    await card.screenshot({ path: `${ART}/a6_evolved_green.png` });
  }
});

test("A3 rush is yellow the turn played, green next; storm is green", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const rush = await importDeck(page, "rush.json", { "10631110": 40 });
  await page.locator("#redDeckSelect").selectOption(rush);
  await startGame(page, { seed: "1", first: "b", deckA: rush, deckB: rush });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await playCard(page, "10631110");
  const spawn = page.locator("#blueBoard .card[data-card='10631110']").first();
  await expect(spawn).toHaveClass(/rush-glow/);
  await expect(spawn).not.toHaveClass(/can-attack/);
  await mkdir(ART, { recursive: true });
  await spawn.screenshot({ path: `${ART}/a3_rush_yellow.png` });
  await endTurnApply(page);
  await endTurnApply(page);
  await expect(spawn).toHaveClass(/can-attack/);
  await expect(spawn).not.toHaveClass(/rush-glow/);
  await spawn.screenshot({ path: `${ART}/a3_rush_green.png` });

  const storm = await importDeck(page, "barbaros.json", { "10924110": 40 });
  await page.locator("#redDeckSelect").selectOption(storm);
  await startGame(page, { seed: "2", first: "a", deckA: storm, deckB: storm });
  await confirmMulligans(page);
  await closeDrawer(page);
  await skipToPp(page, 7);
  await playCard(page, "10924110");
  const barb = page.locator("#blueBoard .card[data-card='10924110']").first();
  await expect(barb).toHaveClass(/can-attack/);
  await expect(barb).not.toHaveClass(/rush-glow/);
  await barb.screenshot({ path: `${ART}/a3_storm_green.png` });
});

test("A4 A5 Slice yellow 2/2 vs green 1/2; no E/A/C badge", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "slice-spawn.json", { "10823310": 20, "10631110": 20 });
  await page.locator("#redDeckSelect").selectOption(id);
  await startGame(page, { seed: "3", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);

  async function sliceState() {
    return page.evaluate(() => {
      const info = window.__arena!.handInfo("a") as Array<{
        id: string;
        form: string | null;
        playable: boolean;
        gates: Array<{ kind: string; label?: string; have: number; need: number; met: boolean }>;
      }>;
      const slice = info.find((c) => c.id === "10823310");
      const field = (
        window.__arena!.full() as { players: { a: { field: Array<{ card?: string } | null> } } }
      ).players.a.field.filter(Boolean).length;
      return { slice, field };
    });
  }

  for (let i = 0; i < 24; i++) {
    const st = await sliceState();
    const active = await page.evaluate(
      () => (window.__arena!.full() as { active: string }).active,
    );
    if (st.slice && st.field >= 2 && st.slice.playable && active === "a") break;
    const legal = await page.evaluate(() => window.__arena!.legal() as Array<Record<string, unknown>>);
    const playSpawn = legal.find((a) => "play" in a && (a.play as { card: string }).card === "10631110");
    if (playSpawn && st.field < 2 && active === "a") await applyAction(page, playSpawn);
    else await endTurnApply(page);
  }

  const afterTwo = await sliceState();
  expect(afterTwo.slice, "Slice of Domesticity in A's hand").toBeTruthy();
  expect(afterTwo.field).toBeGreaterThanOrEqual(2);
  expect(afterTwo.slice!.playable).toBeTruthy();
  const gate = afterTwo.slice!.gates.find((g) => g.kind === "countAtLeast");
  expect(gate?.label).toContain("allied cards on the field");
  expect(gate?.have).toBeGreaterThanOrEqual(2);
  expect(gate?.met).toBeTruthy();
  const cardTwo = page.locator("#blueHand .card[data-card='10823310']").first();
  await expect(cardTwo).toHaveClass(/enhance-ready/);
  await expect(cardTwo.locator(".alternate-form-badge")).toHaveCount(0);
  await cardTwo.hover();
  await expect(page.locator("#cardTooltip")).toContainText("Allied cards on the field");
  await expect(page.locator("#cardTooltip")).toContainText("2/2");
  await mkdir(ART, { recursive: true });
  await page.locator("#cardTooltip").screenshot({ path: `${ART}/a5_slice_2_2.png` });
  await cardTwo.screenshot({ path: `${ART}/a5_slice_yellow.png` });

  // 1-ally path: new game, play one spawn only.
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  for (let i = 0; i < 16; i++) {
    const st = await sliceState();
    const active = await page.evaluate(
      () => (window.__arena!.full() as { active: string }).active,
    );
    if (st.slice && st.field === 1 && st.slice.playable && active === "a") break;
    const legal = await page.evaluate(() => window.__arena!.legal() as Array<Record<string, unknown>>);
    const playSpawn = legal.find((a) => "play" in a && (a.play as { card: string }).card === "10631110");
    if (playSpawn && (st.field ?? 0) < 1 && active === "a") await applyAction(page, playSpawn);
    else await endTurnApply(page);
  }
  const one = await sliceState();
  expect(one.slice, "Slice in hand for 1-ally path").toBeTruthy();
  expect(one.field).toBe(1);
  expect(one.slice!.playable).toBeTruthy();
  const gateOne = one.slice!.gates.find((g) => g.kind === "countAtLeast");
  expect(gateOne?.have).toBe(1);
  expect(gateOne?.met).toBeFalsy();
  const cardOne = page.locator("#blueHand .card[data-card='10823310']").first();
  await expect(cardOne).toHaveClass(/playable-glow/);
  await cardOne.hover();
  await expect(page.locator("#cardTooltip")).toContainText("1/2");
  await cardOne.screenshot({ path: `${ART}/a5_slice_1_2.png` });
});

test("A6 evolved stats are green; damaged defense is orange", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "stat-colors.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "b", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await playCard(page, "10631110");
  for (let i = 0; i < 12; i++) {
    const unlocked = await page.evaluate(
      () => window.__arena!.playerInfo("a").evolve_unlocked,
    );
    if (unlocked) break;
    await endTurnApply(page);
  }
  const evo = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
    return legal.find((a) => a.evolve && !a.evolve.super) ?? null;
  });
  expect(evo).toBeTruthy();
  await applyAction(page, evo);
  const card = page.locator("#blueBoard .card[data-card='10631110']").first();
  await expect(card.locator(".card-stats.bottom-left")).toHaveClass(/stat-buffed/);
  await expect(card.locator(".card-stats.bottom-right")).toHaveClass(/stat-buffed/);
  await mkdir(ART, { recursive: true });
  await card.screenshot({ path: `${ART}/a6_evolved_green.png` });
  const atk = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { target: { slot?: number } | "leader" };
    }>;
    return legal.find((a) => a.attack && a.attack.target !== "leader") ?? null;
  });
  expect(atk).toBeTruthy();
  await applyAction(page, atk);
  await expect(card.locator(".card-stats.bottom-left")).toHaveClass(/stat-buffed/);
  await expect(card.locator(".card-stats.bottom-right")).toHaveClass(/stat-damaged/);
  await card.screenshot({ path: `${ART}/a6_damaged_orange.png` });
});

test("A7 faith badge increments after an Enhanced play", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "faith-depths.json", {
    "10624120": 15,
    "90024320": 15,
    "10631110": 10,
  });
  await startGame(page, { seed: "1", first: "b", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  const crest = page.locator("#blueCrests .crest-slot .crest-faith").first();
  await expect(crest).toBeVisible({ timeout: 10_000 });
  const before = Number(await crest.innerText());
  expect(before).toBe(0);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await skipToPp(page, 8);
  for (let i = 0; i < 4; i++) {
    const active = await page.evaluate(
      () => (window.__arena!.full() as { active: string }).active,
    );
    if (active === "a") break;
    await endTurnApply(page);
  }
  await playCard(page, "90024320");
  await expect(crest).not.toHaveText(String(before), { timeout: 5000 });
  expect(Number(await crest.innerText())).toBeGreaterThan(before);
  await mkdir(ART, { recursive: true });
  await page.locator("#blueCrests").screenshot({ path: `${ART}/a7_faith_badge.png` });
});

test("B1 B2 B18 fuse confirm inside modal, labelled partners, fuse chip", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startGame(page, {
    seed: "3",
    first: "a",
    deckA: "rune-mach15",
    deckB: "rune-mach15",
  });
  await confirmMulligans(page);
  await closeDrawer(page);

  async function driveToFuse(side: "a" | "b") {
    for (let i = 0; i < 24; i++) {
      const fuse = await page.evaluate((p) => {
        const legal = window.__arena!.legal() as Array<{
          fuse?: { player: string; host_pos: number };
          end_turn?: unknown;
          mulligan?: unknown;
        }>;
        const act = legal.find((a) => a.fuse && a.fuse.player === p);
        if (act) {
          window.__arena!.apply(act);
          return "fuse";
        }
        const end = legal.find((a) => a.end_turn);
        if (end) {
          window.__arena!.apply(end);
          return "end";
        }
        return "";
      }, side);
      if (fuse === "fuse") return true;
    }
    return false;
  }

  const startedFuse = (await driveToFuse("a")) || (await driveToFuse("b"));
  expect(startedFuse).toBeTruthy();
  const chip = page.locator(".fuse-chip").first();
  if (await chip.count()) {
    await mkdir(ART, { recursive: true });
    await chip.screenshot({ path: `${ART}/b18_fuse_chip.png` });
  }

  const phase = await page.locator("#turnCounter").getAttribute("data-phase");
  if (phase === "choice") {
    const options = page.locator(".choice-option, .hand-zone .card.legal-target");
    await expect(options.first()).toBeVisible();
    const labels = await page.locator(".choice-option").allTextContents();
    for (const t of labels) {
      expect(t).not.toMatch(/^Partner #/);
      expect(t).not.toMatch(/^Hand \d/);
    }
    if (await page.locator(".hand-zone .card.legal-target").count()) {
      await page.locator(".hand-zone .card.legal-target").first().click();
    } else if (await page.locator(".choice-option").count()) {
      await page.locator(".choice-option").first().click();
    }
    const confirm = page.locator(".choice-modal .confirm-targets-btn, .choice-prompt-bar .confirm-targets-btn, #targetingConfirmation .confirm-targets-btn");
    await expect(confirm.first()).toBeVisible();
    const box = await confirm.first().boundingBox();
    expect(box).toBeTruthy();
    const hit = await page.evaluate(({ x, y }) => {
      const el = document.elementFromPoint(x, y);
      return el?.className ?? "";
    }, { x: box!.x + box!.width / 2, y: box!.y + box!.height / 2 });
    expect(hit).toMatch(/confirm|choice-prompt/);
    await mkdir(ART, { recursive: true });
    await page.screenshot({ path: `${ART}/b1_fuse_confirm.png` });
    await confirm.first().click();
  }
});

test("B3 choice click does not swallow the next click", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "sword-pool",
    deckB: "abyss-pool",
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  const played = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === "10704110" || a.play?.card === "10104110");
    // Measured Attunement / similar — fall back to any play that offers a choice.
    if (act) {
      window.__arena!.apply(act);
      return true;
    }
    return false;
  });
  if (await page.locator("#turnCounter").getAttribute("data-phase") === "choice") {
    await page.keyboard.press("Control+z");
  } else if (played) {
    await page.keyboard.press("Control+z");
  }
  const attacker = page.locator("#blueBoard .card.legal-attack, #blueBoard .card.can-attack").first();
  if (await attacker.count()) {
    await attacker.click();
    await expect(page.locator(".pending-cancel-chip")).toBeVisible();
    await page.locator("#blueLeader").click();
    await expect(page.locator(".pending-cancel-chip")).toHaveCount(0);
  }
});

test("B5 mulligan marks stay on the acting hand", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "sword-pool",
    deckB: "abyss-pool",
  });
  await closeDrawer(page);
  const acting = await page.locator("#turnCounter").getAttribute("data-acting");
  const actingHand = acting === "a" ? "#blueHand" : "#redHand";
  const idleHand = acting === "a" ? "#redHand" : "#blueHand";
  await page.locator(`${actingHand} .card`).nth(0).click();
  await page.locator(`${actingHand} .card`).nth(2).click();
  await expect(page.locator(`${actingHand} .card.selected`)).toHaveCount(2);
  await page.locator(`${idleHand} .card`).nth(1).click();
  await expect(page.locator(`${idleHand} .card.selected`)).toHaveCount(0);
  await expect(page.locator(`${actingHand} .card.selected`)).toHaveCount(2);
});

test("B6 drop highlight only on legal attack targets", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startGame(page, {
    seed: "2",
    first: "a",
    deckA: "abyss-pool",
    deckB: "portal-pool",
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  for (let i = 0; i < 8; i++) {
    const ready = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{
        attack?: { target: unknown; attacker_slot: number };
      }>;
      return legal.some((a) => a.attack);
    });
    if (ready) break;
    await endTurnApply(page);
  }
  const atk = page.locator("#blueBoard .card.legal-attack, #blueBoard .card.can-attack, #redBoard .card.legal-attack").first();
  if (!(await atk.count())) return;
  const box = await atk.boundingBox();
  if (!box) return;
  const leader = page.locator("#redLeader");
  const lbox = await leader.boundingBox();
  if (!lbox) return;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(lbox.x + lbox.width / 2, lbox.y + lbox.height / 2, { steps: 8 });
  const highlighted = await leader.evaluate((el) => el.classList.contains("pointer-drop-highlight"));
  const legalLeader = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ attack?: { target: unknown } }>;
    return legal.some((a) => a.attack && a.attack.target === "leader");
  });
  expect(highlighted).toBe(legalLeader);
  await page.mouse.up();
});

test("B7 follower floating combat text", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "fct-rush.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "b", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await playCard(page, "10631110");
  const events = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { target: { slot?: number } | "leader" };
    }>;
    const act = legal.find((a) => a.attack && a.attack.target !== "leader");
    if (!act) return [];
    return window.__arena!.apply(act) as unknown[];
  });
  expect(events.some((e) => e && typeof e === "object" && "damage" in e)).toBeTruthy();
  await expect(page.locator(".floating-combat-text")).toHaveCount(1, { timeout: 4000 });
  await mkdir(ART, { recursive: true }).catch(() => undefined);
  await page.screenshot({ path: `${ART}/b7_follower_fct.png` }).catch(() => undefined);
});

test("B8 destroyed history stays with the owner across End Turn", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startGame(page, {
    seed: "2",
    first: "a",
    deckA: "abyss-pool",
    deckB: "portal-pool",
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  for (let i = 0; i < 24; i++) {
    const before = await page.locator("#blueDestroyedList li, #redDestroyedList li").count();
    const atk = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ attack?: unknown }>;
      return legal.find((a) => a.attack) ?? null;
    });
    if (atk) await applyAction(page, atk);
    else await endTurnApply(page);
    const after = await page.locator("#blueDestroyedList li, #redDestroyedList li").count();
    if (after > before) {
      const snap = await page.evaluate(() => ({
        blue: [...document.querySelectorAll("#blueDestroyedList .hist-label")].map((e) => e.textContent),
        red: [...document.querySelectorAll("#redDestroyedList .hist-label")].map((e) => e.textContent),
      }));
      await page.locator("#historyToggle").click();
      await endTurnApply(page);
      const again = await page.evaluate(() => ({
        blue: [...document.querySelectorAll("#blueDestroyedList .hist-label")].map((e) => e.textContent),
        red: [...document.querySelectorAll("#redDestroyedList .hist-label")].map((e) => e.textContent),
      }));
      expect(again.blue).toEqual(snap.blue);
      expect(again.red).toEqual(snap.red);
      return;
    }
  }
});

test("B9 tap-anywhere cancels pending attack; Cancel chip visible", async ({ page }) => {
  test.setTimeout(60_000);
  await boot(page);
  const id = await importDeck(page, "rush2.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await endTurnApply(page);
  const atk = page.locator("#blueBoard .card.can-attack, #blueBoard .card.legal-attack").first();
  await atk.click();
  await expect(page.locator(".pending-cancel-chip")).toBeVisible();
  await mkdir(ART, { recursive: true });
  await page.screenshot({ path: `${ART}/b9_cancel_chip.png` });
  await page.locator("#blueHand").click({ position: { x: 8, y: 8 } });
  await expect(page.locator(".pending-cancel-chip")).toHaveCount(0);
});

test("B12 mode buttons carry printed 1-based text", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const modes = await importDeck(page, "modes.json", { "10423310": 40 });
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: modes,
    deckB: modes,
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  await skipToPp(page, 5);
  for (let i = 0; i < 16; i++) {
    const played = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const knight = legal.find(
        (a) => a.play?.card === "10423110" || a.play?.card === "10423310",
      );
      if (knight) {
        window.__arena!.apply(knight);
        return true;
      }
      return false;
    });
    if (played || (await page.locator("#turnCounter").getAttribute("data-phase")) === "choice") break;
    const anyPlay = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ play?: unknown }>;
      const act = legal.find((a) => a.play);
      if (act) window.__arena!.apply(act);
      return !!act;
    });
    if (!anyPlay) await endTurnApply(page);
  }
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice", {
    timeout: 15_000,
  });
  const texts = await page.locator(".choice-option").allTextContents();
  expect(texts.length).toBeGreaterThan(0);
  for (const t of texts) {
    expect(t).not.toMatch(/^Mode \d+$/);
    expect(t.length).toBeGreaterThan(8);
  }
  expect(texts.some((t) => /Give the leftmost|Barrier|Recover|Restore/i.test(t))).toBeTruthy();
  await mkdir(ART, { recursive: true });
  await page.locator(".choice-modal").screenshot({ path: `${ART}/b12_mode_text.png` });
});

test("B14 save/load after 200+ actions and invalid file", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await confirmMulligans(page);
  await closeDrawer(page);
  await endTurnApply(page);
  for (let i = 0; i < 205; i++) {
    const toggled = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ bonus_pp?: unknown }>;
      const act = legal.find((a) => a.bonus_pp);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    });
    if (!toggled) break;
  }
  const ply = await page.locator("#turnCounter").innerText();
  expect(Number(ply)).toBeGreaterThan(0);
  await openSettings(page);
  page.once("dialog", (d) => d.accept("ring-pos"));
  await page.locator("#savePositionBtn").click();
  await page.locator("#positionSelect").selectOption({ label: "ring-pos" });
  await page.locator("#loadPositionBtn").click();
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /main|choice/, {
    timeout: 15_000,
  });
  await page.locator("#importPositionInput").setInputFiles({
    name: "bad.json",
    mimeType: "application/json",
    buffer: Buffer.from("{not-a-position"),
  });
  await expect(page.locator("#toastHost")).toBeVisible({ timeout: 5000 });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /main|choice|mulligan/);
});

test("A9 A10 evo buttons stay enabled after unlock; countdown while locked", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const id = await importDeck(page, "evo-cd.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);

  type Info = {
    evolve_unlocked: boolean;
    super_evolve_unlocked: boolean;
    evolve_unlock_in: number;
    super_evolve_unlock_in: number;
  };

  async function snap() {
    return page.evaluate(() => {
      const btn = (id: string) => {
        const el = document.getElementById(id) as HTMLButtonElement;
        return {
          disabled: el.disabled,
          badge: el.querySelector(".evo-unlock-badge")?.textContent ?? null,
          title: el.title,
          label: el.querySelector(".evo-btn-label")?.textContent ?? el.textContent ?? "",
        };
      };
      return {
        active: (window.__arena!.full() as { active: string }).active,
        a: window.__arena!.playerInfo("a"),
        b: window.__arena!.playerInfo("b"),
        blueEvo: btn("blueNormalEvo"),
        blueSuper: btn("blueSuperEvo"),
        redEvo: btn("redNormalEvo"),
        redSuper: btn("redSuperEvo"),
      };
    });
  }

  const t1 = await snap();
  expect(t1.a.evolve_unlocked).toBeFalsy();
  expect(t1.a.super_evolve_unlocked).toBeFalsy();
  expect(t1.a.evolve_unlock_in).toBe(4);
  expect(t1.a.super_evolve_unlock_in).toBe(6);
  expect(t1.b.evolve_unlock_in).toBe(4);
  expect(t1.b.super_evolve_unlock_in).toBe(6);
  expect(t1.blueEvo.disabled).toBeTruthy();
  expect(t1.blueSuper.disabled).toBeTruthy();
  expect(t1.redEvo.disabled).toBeTruthy();
  expect(t1.redSuper.disabled).toBeTruthy();
  expect(t1.blueEvo.badge).toBe("4");
  expect(t1.blueSuper.badge).toBe("6");
  expect(t1.blueEvo.title).toBe("unlocks in 4 turns");
  await mkdir(ART, { recursive: true });
  await page.locator("#blueLeader").screenshot({ path: `${ART}/a10_evo_countdown_locked.png` });

  let aUnlockedAt: number | null = null;
  for (let ply = 0; ply < 16; ply++) {
    const before = await snap();
    await endTurnApply(page);
    const after = await snap();
    const started = after.active as "a" | "b";
    const idle = started === "a" ? "b" : "a";
    if (before[started].evolve_unlock_in > 0) {
      expect(after[started].evolve_unlock_in).toBe(before[started].evolve_unlock_in - 1);
    } else {
      expect(after[started].evolve_unlock_in).toBe(0);
    }
    expect(after[idle].evolve_unlock_in).toBe(before[idle].evolve_unlock_in);
    expect(after[idle].super_evolve_unlock_in).toBe(before[idle].super_evolve_unlock_in);

    const firstBtn = started === "a" ? after.blueEvo : after.redEvo;
    const firstSuper = started === "a" ? after.blueSuper : after.redSuper;
    const info = after[started] as Info;
    if (info.evolve_unlocked) {
      expect(firstBtn.disabled).toBeFalsy();
      expect(firstBtn.badge).toBeNull();
      if (aUnlockedAt == null && started === "a") aUnlockedAt = ply;
    } else {
      expect(firstBtn.disabled).toBeTruthy();
      expect(firstBtn.badge).toBe(String(info.evolve_unlock_in));
    }
    if (info.super_evolve_unlocked) {
      expect(firstSuper.disabled).toBeFalsy();
      expect(firstSuper.badge).toBeNull();
    } else {
      expect(firstSuper.disabled).toBeTruthy();
      expect(firstSuper.badge).toBe(String(info.super_evolve_unlock_in));
    }
    if (after.a.evolve_unlocked && after.b.evolve_unlocked) break;
  }

  expect(aUnlockedAt).not.toBeNull();
  const unlockedShot = await snap();
  expect(unlockedShot.blueEvo.disabled).toBeFalsy();
  await page.locator("#blueLeader").screenshot({ path: `${ART}/a9_evo_unlocked.png` });

  for (let i = 0; i < 6; i++) {
    const evo = await page.evaluate(() => {
      const legal = window.__arena!.legal() as Array<{ evolve?: { super: boolean } }>;
      return legal.find((a) => a.evolve && !a.evolve.super) ?? null;
    });
    if (evo) {
      await applyAction(page, evo);
      const mid = await snap();
      expect(mid.blueEvo.disabled).toBeFalsy();
      if (mid.a.super_evolve_unlocked) expect(mid.blueSuper.disabled).toBeFalsy();
      expect(mid.blueEvo.label).toMatch(/Evo \(\d\)/);
      await page.locator("#blueLeader").screenshot({ path: `${ART}/a9_evo_after_use.png` });
    }
    await endTurnApply(page);
    const later = await snap();
    expect(later.blueEvo.disabled).toBeFalsy();
    expect(later.redEvo.disabled).toBeFalsy();
  }
});

test.describe("touch", () => {
  test.use({ hasTouch: true, viewport: { width: 1024, height: 768 } });
  test("C touch drag-to-play and drag-attack at 1024x768", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "touch-rush.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  const hand = page.locator("#blueHand .card.legal-play").first();
  await expect(hand).toBeVisible();
  const board = page.locator("#blueBoard");
  const h = await hand.boundingBox();
  const b = await board.boundingBox();
  expect(h && b).toBeTruthy();
  await page.mouse.move(h!.x + h!.width / 2, h!.y + h!.height / 2);
  await page.mouse.down();
  await page.mouse.move(b!.x + b!.width / 2, b!.y + b!.height / 2, { steps: 12 });
  await page.mouse.up();
  await expect(page.locator("#blueBoard .card")).toHaveCount(1, { timeout: 8000 });
  await endTurnApply(page);
  await endTurnApply(page);
  const atk = page.locator("#blueBoard .card.can-attack, #blueBoard .card.legal-attack").first();
  if (await atk.count()) {
    const ab = await atk.boundingBox();
    const enemy = page.locator("#redLeader");
    const eb = await enemy.boundingBox();
    if (ab && eb) {
      await page.mouse.move(ab.x + ab.width / 2, ab.y + ab.height / 2);
      await page.mouse.down();
      await page.mouse.move(eb.x + eb.width / 2, eb.y + eb.height / 2, { steps: 10 });
      await page.mouse.up();
    }
  }
  });
});
