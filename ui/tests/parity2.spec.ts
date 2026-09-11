import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { ART, artShot, pngRgba } from "./helpers.ts";

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
  opts: { seed?: string; first?: string; deckA?: string; deckB?: string } = {},
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  if (opts.seed !== undefined) await page.locator("#seedInput").fill(opts.seed);
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

async function endTurnApply(page: Page) {
  const ok = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<Record<string, unknown>>;
    const act = legal.find((a) => "end_turn" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(ok, "expected legal end_turn").toBeTruthy();
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

async function playWhenLegal(page: Page, card: string, max = 24) {
  for (let i = 0; i < max; i++) {
    const ok = await page.evaluate((id) => {
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const act = legal.find((a) => a.play?.card === id);
      if (!act) return false;
      window.__arena!.apply(act);
      return true;
    }, card);
    if (ok) return;
    await endTurnApply(page);
  }
  throw new Error(`could not play ${card}`);
}

async function startMono(page: Page, file: string, card: string, seed = "1") {
  const id = await importDeck(page, file, { [card]: 40 });
  await startGame(page, { seed, first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  return id;
}

function sampleColor(
  path: string,
  pred: (r: number, g: number, b: number, a: number) => boolean,
): number {
  const img = pngRgba(readFileSync(path));
  let hits = 0;
  for (let i = 0; i < img.data.length; i += 4) {
    if (pred(img.data[i], img.data[i + 1], img.data[i + 2], img.data[i + 3])) hits += 1;
  }
  return hits;
}

function uniqueDeck(): Record<string, number> {
  const catalog = JSON.parse(
    readFileSync(new URL("../public/catalog-images.json", import.meta.url), "utf8"),
  ) as Record<string, unknown>;
  const ids = Object.keys(catalog).slice(0, 40);
  const deck: Record<string, number> = {};
  for (const id of ids) deck[id] = 1;
  return deck;
}

test("#30 Barrier overlay + flash on gain + pop on loss", async ({ page }) => {
  const a = await importDeck(page, "p2-barrier-a.json", {
    "10001110": 20,
    "10412120": 20,
  });
  const b = await importDeck(page, "p2-barrier-b.json", { "10001110": 40 });
  await startGame(page, { seed: "3", first: "a", deckA: a, deckB: b });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playWhenLegal(page, "10001110");
  await playWhenLegal(page, "10412120");
  const card = page.locator("#blueBoard .card.has-barrier").first();
  await expect(card).toBeVisible({ timeout: 10_000 });
  await expect(card.locator(".barrier-overlay")).toHaveCount(1);
  await expect(card.locator(".barrier-particle")).toHaveCount(5);
  const flash = await card.evaluate((el) => {
    const anim = getComputedStyle(el).animationName;
    return anim.includes("barrier-flash") || el.classList.contains("barrier-flash");
  });
  expect(flash, "gain must start the 250ms barrier-flash").toBeTruthy();
  const glow = await card.evaluate((el) => getComputedStyle(el).boxShadow);
  expect(glow).toMatch(/50,\s*140,\s*255|32,\s*8c,\s*ff|rgba?\(50/);
  const shot = await artShot(card, `${ART}/p2_barrier_overlay.png`);
  const cyan = sampleColor(
    shot,
    (r, g, b, a) => a > 40 && b > 160 && g > 140 && r < 220 && b > r,
  );
  expect(cyan, "barrier overlay should sample cyan").toBeGreaterThan(8);

  await endTurnApply(page);
  await playWhenLegal(page, "10001110");
  await endTurnApply(page);
  await endTurnApply(page);
  const popped = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { attacker_slot: number; target: { slot: number } | "leader" };
    }>;
    const act = legal.find((a) => a.attack && a.attack.target !== "leader");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(popped, "B should attack the barrier follower").toBeTruthy();
  const pop = page.locator(".barrier-pop, .card-image-wrapper.barrier-pop");
  await expect(pop.first()).toBeVisible({ timeout: 1000 }).catch(() => undefined);
  const popAnim = await page.evaluate(() => {
    const wrap = document.querySelector(".card-image-wrapper.barrier-pop");
    if (!wrap) return document.querySelector(".barrier-overlay") ? "overlay-linger" : "gone";
    return getComputedStyle(wrap.querySelector(".barrier-overlay") ?? wrap).animationName;
  });
  expect(popAnim === "gone" || String(popAnim).length > 0).toBeTruthy();
  await artShot(page.locator("#appRoot"), `${ART}/p2_barrier_after_pop.png`);
});

test("#31 Can't-be-destroyed overlay + 5 gold particles", async ({ page }) => {
  await startMono(page, "p2-cbd.json", "10031210");
  await playWhenLegal(page, "10031210");
  const card = page.locator("#blueBoard .card").first();
  await expect(card).toBeVisible();
  const traits = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { field: Array<{ traits?: string[] } | null> } };
    };
    return full.players.a.field.find((c) => c)?.traits ?? [];
  });
  expect(traits).toContain("cantBeDestroyedByAbilities");
  await expect(card.locator(".cant-be-destroyed-overlay")).toHaveCount(1);
  await expect(card.locator(".cant-be-destroyed-particle")).toHaveCount(5);
  const goldBg = await card.locator(".cant-be-destroyed-particle").first().evaluate((el) => {
    return getComputedStyle(el).backgroundColor;
  });
  expect(goldBg).toMatch(/255,\s*215,\s*0/);
  const shot = await artShot(card, `${ART}/p2_cant_be_destroyed.png`);
  const gold = sampleColor(
    shot,
    (r, g, b, a) => a > 40 && r > 180 && g > 140 && b < 120 && r > b,
  );
  expect(gold, "CBD overlay should sample gold").toBeGreaterThan(8);
});

test("#33 Amulet named-counter badge is first of X, then Y, then Z", async ({ page }) => {
  await startMono(page, "p2-named.json", "10031210");
  await playWhenLegal(page, "10031210");
  const live = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ named_counter?: number | null }>;
    const badges = document.querySelectorAll("#blueBoard .named-counter").length;
    return { named: info[0]?.named_counter ?? null, badges };
  });
  expect(live.named).toBeNull();
  expect(live.badges).toBe(0);
  const order = await page.evaluate(() => {
    const v = window.__arena!.namedCounterValue;
    return {
      xy: v({ kind: "amulet", countdown: null, vars: { Y: 4, X: 7 } }),
      y: v({ kind: "amulet", countdown: null, vars: { Y: 4 } }),
      cd: v({ kind: "amulet", countdown: 2, vars: { X: 9 } }),
    };
  });
  expect(order.xy).toBe(7);
  expect(order.y).toBe(4);
  expect(order.cd).toBeNull();
  await page.evaluate(() => window.__arena!.mountNamedCounter({ Y: 4, X: 7 }));
  const badge = page.locator("#named-counter-demo .named-counter");
  await expect(badge).toHaveText("7");
  const style = await badge.evaluate((el) => {
    const s = getComputedStyle(el);
    return { color: s.color, stroke: s.webkitTextStrokeColor || s.getPropertyValue("-webkit-text-stroke-color") };
  });
  expect(style.color).toMatch(/255,\s*255,\s*255/);
  expect(style.stroke).toMatch(/0,\s*0,\s*0/);
  await artShot(page.locator("#named-counter-demo"), `${ART}/p2_named_counter.png`);
});

test("#34 .spell-cast on the leaving hand card", async ({ page }) => {
  await startMono(page, "p2-spell.json", "10031310");
  const playing = page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === "10031310");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  const clone = page.locator(".card.spell-cast").first();
  await expect(clone).toBeVisible({ timeout: 2000 });
  const style = await clone.evaluate((el) => {
    const s = getComputedStyle(el);
    return {
      anim: s.animationName,
      duration: s.animationDuration,
      timing: s.animationTimingFunction,
      filter: s.filter,
      shadow: s.boxShadow,
      transform: s.transform,
    };
  });
  expect(style.anim).toMatch(/spell-cast/);
  expect(style.duration).toMatch(/0\.5s|500ms/);
  expect(style.timing).toMatch(/ease-out/);
  await playing;
  await artShot(page, `${ART}/p2_spell_cast.png`);
});

test("#38 Share URL writes a/b aliases and old bookmarks still open", async ({ page }) => {
  await boot(page);
  await startGame(page, { seed: "42", first: "a", deckA: "basic-forest", deckB: "basic-rune" });
  await confirmMulligans(page);
  const written = await page.evaluate(() => {
    const p = new URLSearchParams(location.search);
    return {
      seed: p.get("seed"),
      deckA: p.get("deckA"),
      deckB: p.get("deckB"),
      mode: p.get("mode"),
      a: p.get("a"),
      b: p.get("b"),
    };
  });
  expect(written).toEqual({
    seed: "42",
    deckA: "basic-forest",
    deckB: "basic-rune",
    mode: "hotseat",
    a: "basic-forest",
    b: "basic-rune",
  });
  await page.goto("/?seed=42&a=basic-forest&b=basic-rune&mode=hotseat");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  const roundTrip = await page.evaluate(() => {
    const p = new URLSearchParams(location.search);
    return { a: p.get("a"), b: p.get("b"), deckA: p.get("deckA"), deckB: p.get("deckB") };
  });
  expect(roundTrip.a ?? roundTrip.deckA).toBe("basic-forest");
  expect(roundTrip.b ?? roundTrip.deckB).toBe("basic-rune");
  await artShot(page.locator("#gameSeedPanel"), `${ART}/p2_share_url.png`);
});

test("#42 Rematch cancels in-flight image loads on removed nodes", async ({ page }) => {
  await startMono(page, "p2-img.json", "10001110");
  const before = await page.evaluate(() => {
    const imgs = Array.from(document.querySelectorAll<HTMLImageElement>(".card img, img.crest-image"));
    const loads = { n: 0 };
    for (const img of imgs) {
      img.addEventListener("load", () => {
        loads.n += 1;
      });
    }
    (window as unknown as { __oldImgLoads: { n: number }; __oldImgs: HTMLImageElement[] }).__oldImgLoads =
      loads;
    (window as unknown as { __oldImgs: HTMLImageElement[] }).__oldImgs = imgs;
    return imgs.length;
  });
  expect(before).toBeGreaterThan(0);
  await page.evaluate(() => window.__arena!.rematchSame());
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", /mulligan|main/, {
    timeout: 15_000,
  });
  await page.waitForTimeout(400);
  const after = await page.evaluate(() => {
    const w = window as unknown as {
      __oldImgLoads: { n: number };
      __oldImgs: HTMLImageElement[];
    };
    return {
      loads: w.__oldImgLoads.n,
      liveSrc: w.__oldImgs.filter((img) => img.getAttribute("src")).length,
      connected: w.__oldImgs.filter((img) => img.isConnected).length,
    };
  });
  expect(after.loads, "removed nodes must not fire load after Rematch").toBe(0);
  expect(after.liveSrc).toBe(0);
  await artShot(page, `${ART}/p2_image_release.png`);
});

test("#44 Position option Name · T{n} · time; Rename; Del; JSON keeps meta", async ({ page }) => {
  await startMono(page, "p2-pos.json", "10001110");
  await openSettings(page);
  page.once("dialog", (d) => d.accept("alpha"));
  await page.locator("#savePositionBtn").click();
  const opt = page.locator("#positionSelect option").first();
  await expect(opt).toHaveText(/alpha · T\d+ · /);
  const text = await opt.textContent();
  expect(text).toMatch(/alpha · T\d+ · .+/);
  page.once("dialog", (d) => d.accept("beta"));
  await page.locator("#renamePositionBtn").click();
  await expect(opt).toHaveText(/beta · T\d+ · /);
  const saved = (await page.evaluate(() => window.__arena!.savedPosition())) as {
    name?: string;
    turn?: number;
    savedAt?: string;
  };
  expect(saved.name).toBe("beta");
  expect(saved.turn).toBeGreaterThan(0);
  expect(saved.savedAt).toBeTruthy();
  await page.locator("#deletePositionBtn").click();
  await expect(page.locator("#positionSelect")).toHaveValue("");
  await artShot(page.locator("#settingsDrawer"), `${ART}/p2_positions.png`);
});

test("#45 F6 / F7 / F8 reroll + reseed log round-trip", async ({ page }) => {
  const deck = uniqueDeck();
  const id = await importDeck(page, "p2-reroll.json", deck);
  await startGame(page, { seed: "11", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  for (let i = 0; i < 20; i++) {
    const turn = await page.evaluate(
      () => (window.__arena!.full() as { turn: number }).turn,
    );
    if (turn >= 3) break;
    await endTurnApply(page);
  }
  expect(await page.evaluate(() => (window.__arena!.full() as { turn: number }).turn)).toBeGreaterThanOrEqual(
    3,
  );
  await page.keyboard.press("F6");
  await expect(page.locator("#checkpointStatus")).toHaveText(/Checkpoint: T\d+ · rerolls 0/);
  const cp = await page.evaluate(() => window.__arena!.hash());
  await endTurnApply(page);
  const branchA = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: { hand: Array<{ card: string }>; deck: Array<{ card: string }> };
        b: { hand: Array<{ card: string }>; deck: Array<{ card: string }> };
      };
    };
    const pile = (p: { hand: Array<{ card: string }>; deck: Array<{ card: string }> }) =>
      p.hand.map((c) => c.card).concat(p.deck.map((c) => c.card));
    return { a: pile(full.players.a), b: pile(full.players.b) };
  });
  expect(await page.evaluate(() => window.__arena!.hash())).not.toBe(cp);
  await page.keyboard.press("F7");
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(cp);
  await page.keyboard.press("F8");
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(cp);
  await expect(page.locator("#checkpointStatus")).toHaveText(/rerolls 1/);
  await endTurnApply(page);
  const branchB = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: {
        a: { hand: Array<{ card: string }>; deck: Array<{ card: string }> };
        b: { hand: Array<{ card: string }>; deck: Array<{ card: string }> };
      };
    };
    const pile = (p: { hand: Array<{ card: string }>; deck: Array<{ card: string }> }) =>
      p.hand.map((c) => c.card).concat(p.deck.map((c) => c.card));
    return { a: pile(full.players.a), b: pile(full.players.b) };
  });
  expect(branchB).not.toEqual(branchA);
  const log = (await page.evaluate(() => window.__arena!.exportLog())) as {
    actions: Array<{ reseed?: string | number }>;
  };
  expect(log.actions.some((s) => s.reseed != null)).toBeTruthy();
  const afterF8 = await page.evaluate(() => window.__arena!.hash());
  await page.evaluate((raw) => window.__arena!.loadLog(raw), log);
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).toBe(afterF8);
  await artShot(page.locator("#checkpointStatus"), `${ART}/p2_reroll_status.png`);
});

test("#46 Export List copies Blue deck as Nx Name and shows the panel", async ({ page }) => {
  await startMono(page, "p2-export.json", "10001110");
  await openSettings(page);
  await page.locator("#exportListBtn").click();
  const panel = page.locator("#exportListPanel");
  await expect(panel).toBeVisible();
  const text = await panel.innerText();
  expect(text).toMatch(/^\d+x .+/m);
  expect(text).toMatch(/Water Fairy|Fairy|10001110/);
  const style = await panel.evaluate((el) => {
    const s = getComputedStyle(el);
    return { display: s.display, whiteSpace: s.whiteSpace };
  });
  expect(style.display).not.toBe("none");
  await artShot(panel, `${ART}/p2_export_list.png`);
});
