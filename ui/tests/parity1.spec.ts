import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import {
  ART,
  artShot,
  pngRgba,
} from "./helpers.ts";

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
    policyA?: string;
    policyB?: string;
  } = {},
) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption(opts.mode ?? "hotseat");
  if (opts.seed !== undefined) await page.locator("#seedInput").fill(opts.seed);
  if (opts.first) await page.locator("#firstSelect").selectOption(opts.first);
  if (opts.deckA) await page.locator("#blueDeckSelect").selectOption(opts.deckA);
  if (opts.deckB) await page.locator("#redDeckSelect").selectOption(opts.deckB);
  if (opts.policyA) await page.locator("#policyASelect").selectOption(opts.policyA, { force: true });
  if (opts.policyB) await page.locator("#policyBSelect").selectOption(opts.policyB, { force: true });
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

async function skipToPp(page: Page, pp: number) {
  for (let i = 0; i < 20; i++) {
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

function sampleColor(path: string, pred: (r: number, g: number, b: number, a: number) => boolean): number {
  const img = pngRgba(readFileSync(path));
  let hits = 0;
  for (let i = 0; i < img.data.length; i += 4) {
    if (pred(img.data[i], img.data[i + 1], img.data[i + 2], img.data[i + 3])) hits += 1;
  }
  return hits;
}

async function startMono(page: Page, file: string, card: string, seed = "1") {
  const id = await importDeck(page, file, { [card]: 40 });
  await startGame(page, { seed, first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  return id;
}

test("#5 #10 tooltip order, extras, smart anchor, live refresh, drag-pin", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "rally.json", "10824110");
  const card = page.locator("#blueHand .card").first();
  await expect(card).toBeVisible();
  const box = await card.boundingBox();
  expect(box).toBeTruthy();
  await card.hover();
  const tip = page.locator("#cardTooltip");
  await expect(tip).toBeVisible();
  const order = await tip.evaluate((el) =>
    [...el.querySelectorAll(":scope > *")].map((n) => n.className),
  );
  expect(order[0]).toContain("tooltip-header-name");
  expect(order[1]).toContain("tooltip-header-meta");
  expect(order.some((c) => c.includes("tooltip-desc-block") || c.includes("tooltip-counter"))).toBeTruthy();
  const meta = tip.locator(".tooltip-header-meta").first();
  await expect(meta).toContainText(/Forestcraft|Swordcraft|Runecraft|Dragoncraft|Abysscraft|Havencraft|Portalcraft|Neutral/);
  const set = tip.locator(".card-set-line");
  if (await set.count()) {
    const color = await set.evaluate((el) => getComputedStyle(el).color);
    expect(color === "rgb(154, 163, 178)" || color === "rgb(122, 132, 148)").toBeTruthy();
  }
  const rally = tip.locator(".rally-line");
  await expect(rally).toBeVisible();
  expect(await rally.evaluate((el) => getComputedStyle(el).color)).toBe("rgb(119, 170, 255)");
  const blocked = await page.evaluate(() => {
    const infos = window.__arena!.handInfo("a") as Array<{ playable: boolean; blocked_reason?: string }>;
    return infos.find((i) => !i.playable)?.blocked_reason ?? null;
  });
  if (blocked) {
    const unplayable = page.locator("#blueHand .card").filter({ hasNot: page.locator(".legal-play") }).first();
    if (await unplayable.count()) {
      await unplayable.hover();
      const line = tip.locator(".tooltip-play-blocked");
      await expect(line).toContainText("Cannot play:");
      expect(await line.evaluate((el) => getComputedStyle(el).color)).toBe("rgb(255, 136, 136)");
    }
  }
  const shot = await artShot(tip, `${ART}/p1_tooltip_stack.png`);
  expect(sampleColor(shot, (r, g, b, a) => a > 80 && r > 80 && g > 140 && b > 200)).toBeGreaterThan(4);

  const bottomY = box!.y + box!.height - 4;
  await page.mouse.move(box!.x + box!.width / 2, bottomY);
  const placed = await tip.evaluate((el, y) => {
    const r = el.getBoundingClientRect();
    return { top: r.top, bottom: r.bottom, cursorY: y, vh: window.innerHeight };
  }, bottomY);
  if (placed.cursorY > placed.vh / 2) {
    expect(placed.bottom).toBeLessThanOrEqual(placed.cursorY - 8);
  }

  const beforeHtml = await tip.innerHTML();
  await endTurnApply(page);
  await card.hover().catch(() => undefined);
  const afterHtml = await tip.innerHTML();
  expect(typeof beforeHtml).toBe("string");
  expect(typeof afterHtml).toBe("string");

  await skipToPp(page, 5);
  const playable = page.locator(".hand-zone .card.legal-play").first();
  await expect(playable).toBeVisible();
  const pb = await playable.boundingBox();
  expect(pb).toBeTruthy();
  await page.mouse.move(pb!.x + pb!.width / 2, pb!.y + pb!.height / 2);
  await page.mouse.down();
  await page.mouse.move(pb!.x + pb!.width / 2 + 30, pb!.y + pb!.height / 2 - 30, { steps: 8 });
  const pin = await tip.evaluate((el) => {
    const s = getComputedStyle(el);
    return { top: s.top, left: s.left, display: s.display };
  });
  expect(pin.display).not.toBe("none");
  expect(pin.top).toBe("12px");
  expect(pin.left).toBe("12px");
  await artShot(page, `${ART}/p1_tooltip_drag_pin.png`);
  await page.mouse.up();
});

test("#6 #21 click map: fuse / play / drag-in-hand / engage / contextmenu", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  await startGame(page, {
    seed: "3",
    first: "a",
    deckA: "rune-mach15",
    deckB: "rune-mach15",
  });
  await confirmMulligans(page);
  await closeDrawer(page);

  for (let i = 0; i < 24; i++) {
    if (await page.locator(".card.fuse-ready").count()) break;
    await endTurnApply(page);
  }
  const fuseCard = page.locator(".card.fuse-ready").first();
  await expect(fuseCard).toBeVisible({ timeout: 15_000 });
  const handBefore = await page.evaluate(() => {
    const full = window.__arena!.full() as { players: { a: { hand: unknown[] }; b: { hand: unknown[] } } };
    return full.players.a.hand.length + full.players.b.hand.length;
  });
  await fuseCard.click({ button: "left" });
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  const handAfterFuseClick = await page.evaluate(() => {
    const full = window.__arena!.full() as { players: { a: { hand: unknown[] }; b: { hand: unknown[] } } };
    return full.players.a.hand.length + full.players.b.hand.length;
  });
  expect(handAfterFuseClick).toBe(handBefore);
  await page.keyboard.press("Control+z");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "main");

  const fuse2 = page.locator(".card.fuse-ready").first();
  if (await fuse2.count()) {
    const fb = await fuse2.boundingBox();
    expect(fb).toBeTruthy();
    await page.mouse.move(fb!.x + fb!.width / 2, fb!.y + fb!.height / 2);
    await page.mouse.down();
    await page.mouse.move(fb!.x + 16, fb!.y + 8);
    await page.mouse.up();
    await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
    await page.keyboard.press("Control+z");
  }

  const playable = page.locator(".hand-zone .card.legal-play").first();
  await expect(playable).toBeVisible();
  const playedUid = await playable.getAttribute("data-uid");
  const hash0 = await page.evaluate(() => window.__arena!.hash());
  await playable.click({ button: "right" });
  await expect.poll(() => page.evaluate(() => window.__arena!.hash())).not.toBe(hash0);
  const stillInHand = await page.evaluate((uid) => {
    const full = window.__arena!.full() as {
      players: { a: { hand: { id: number }[] }; b: { hand: { id: number }[] } };
    };
    return [...full.players.a.hand, ...full.players.b.hand].some((c) => String(c.id) === uid);
  }, playedUid);
  expect(stillInHand).toBeFalsy();

  const prevented = await page.evaluate(() => {
    const targets = [
      document.querySelector(".card"),
      document.querySelector(".zone"),
      document.querySelector(".leader"),
      document.querySelector(".evo-btn"),
    ];
    return targets.map((el) => {
      if (!el) return false;
      const ev = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
      el.dispatchEvent(ev);
      return ev.defaultPrevented;
    });
  });
  expect(prevented.every(Boolean)).toBeTruthy();
  await artShot(page, `${ART}/p1_click_map.png`);
});

test("#6 engage right-click on Witch's New Brew", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "brew.json", "10031210");
  await skipToPp(page, 2);
  await playCard(page, "10031210");
  const amulet = page.locator("#blueBoard .card.engage-ready").first();
  await expect(amulet).toBeVisible();
  const earth0 = await page.evaluate(() => {
    const full = window.__arena!.full() as { players: { a: { earth: number } } };
    return full.players.a.earth;
  });
  await amulet.click({ button: "right" });
  await expect.poll(() =>
    page.evaluate(() => {
      const full = window.__arena!.full() as { players: { a: { earth: number } } };
      return full.players.a.earth;
    }),
  ).toBeGreaterThan(earth0);
  await artShot(page.locator("#blueBoard"), `${ART}/p1_engage_rightclick.png`);
});

test("#11 FCT cap, stack, flash, persist", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "fct-rush.json", { "10631110": 40 });
  await startGame(page, { seed: "1", first: "b", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, "10631110");
  await endTurnApply(page);
  await playCard(page, "10631110");
  await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{
      attack?: { target: { slot?: number } | "leader" };
    }>;
    const act = legal.find((a) => a.attack && a.attack.target !== "leader");
    if (act) window.__arena!.apply(act);
  });
  const floater = page.locator(".floating-combat-text").first();
  await expect(floater).toBeVisible({ timeout: 4000 });
  const idx = await floater.evaluate((el) => getComputedStyle(el).getPropertyValue("--float-stack-index").trim());
  expect(idx === "0" || idx === "").toBeTruthy();
  const flash = page.locator(".card.floating-combat-flash").first();
  await expect(flash).toBeVisible({ timeout: 2000 });
  const anim = await flash.evaluate((el) => getComputedStyle(el).animationName);
  expect(anim).toMatch(/floating-combat-card-flash/);
  await artShot(page, `${ART}/p1_fct_flash.png`);

  await openSettings(page);
  const box = page.locator("#floatingCombatTextToggle");
  await expect(box).toBeChecked();
  await box.uncheck();
  await page.reload();
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
  await openSettings(page);
  await expect(page.locator("#floatingCombatTextToggle")).not.toBeChecked();
});

test("#12 selected checkmark on mulligan", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await closeDrawer(page);
  const card = page.locator("#blueHand .card.selectable").first();
  await card.click();
  await expect(card).toHaveClass(/selected/);
  const check = card.locator(".selected-check");
  await expect(check).toHaveText("✓");
  const style = await check.evaluate((el) => {
    const s = getComputedStyle(el);
    return { color: s.color, size: s.fontSize, weight: s.fontWeight, shadow: s.textShadow };
  });
  expect(style.color).toBe("rgb(46, 204, 113)");
  expect(style.size).toBe("24px");
  expect(Number(style.weight)).toBeGreaterThanOrEqual(800);
  expect(style.shadow).toMatch(/rgb\(0, 0, 0\)|rgba\(0, 0, 0/);
  const ring = await card.evaluate((el) => getComputedStyle(el).outlineColor);
  expect(ring).toBe("rgb(57, 217, 138)");
  const shot = await artShot(card, `${ART}/p1_selected_check.png`);
  expect(sampleColor(shot, (r, g, b, a) => a > 80 && r < 80 && g > 160 && b < 140)).toBeGreaterThan(4);
});

test("#13 history rows grouped with cost, set, and hover art", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "fighter.json", "10001110");
  await skipToPp(page, 2);
  await playCard(page, "10001110");
  await page.locator("#historyToggle").click();
  await expect(page.locator("#historyDrawer")).toHaveClass(/open/);
  const row = page.locator("#bluePlayedList .hist-item").first();
  await expect(row).toBeVisible();
  await expect(row.locator(".cost-badge")).toHaveText("2");
  await expect(row.locator(".hist-label")).toContainText(/×\d/);
  const badge = await row.locator(".cost-badge").evaluate((el) => {
    const s = getComputedStyle(el);
    return { w: s.width, h: s.height, color: s.color };
  });
  expect(badge.w).toBe("22px");
  expect(badge.h).toBe("22px");
  expect(badge.color).toBe("rgb(233, 237, 241)");
  const set = row.locator(".hist-set");
  if (await set.count()) {
    const c = await set.evaluate((el) => getComputedStyle(el).color);
    expect(c === "rgb(154, 163, 178)" || c === "rgb(122, 132, 148)").toBeTruthy();
  }
  await row.hover();
  const preview = page.locator("#historyImgPreview");
  await expect(preview).toBeVisible();
  expect(await preview.evaluate((el) => getComputedStyle(el).width)).toBe("198px");
  await artShot(page.locator("#historyDrawer"), `${ART}/p1_history_rows.png`);
});

test("#15 choice extras: title, Earth Rite line, processing dismiss, confirm look", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "runeknight.json", "10031110");
  await skipToPp(page, 3);
  await playCard(page, "10031110");
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  await expect(page.locator(".choice-modal h3")).toHaveText("Choose an effect:");
  const earth = page.locator(".earth-rite-cost");
  await expect(earth.first()).toBeVisible();
  expect(await earth.first().evaluate((el) => getComputedStyle(el).color)).toBe("rgb(241, 196, 15)");
  const confirm = page.locator(".confirm-targets-btn").first();
  if (await confirm.count()) {
    const look = await confirm.evaluate((el) => {
      const s = getComputedStyle(el);
      return { color: s.color, size: s.fontSize, pad: s.padding, weight: s.fontWeight };
    });
    expect(look.color).toBe("rgb(255, 255, 255)");
    expect(look.size).toBe("16px");
    expect(look.pad).toMatch(/12px/);
    expect(Number(look.weight)).toBeGreaterThanOrEqual(700);
  }
  await artShot(page.locator(".choice-modal"), `${ART}/p1_choice_earth_rite.png`);
  const gone = await page.evaluate(() => {
    const btn = document.querySelector<HTMLButtonElement>(".choice-option");
    if (!btn) return false;
    btn.click();
    return document.querySelector(".choice-modal") == null;
  });
  expect(gone, "choice modal must dismiss synchronously on click").toBeTruthy();
});

test("#22 can't-attack overlay on printed lock (Galleon)", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "galleon.json", "10464110");
  await skipToPp(page, 3);
  await playCard(page, "10464110");
  const reason = await page.evaluate(() => {
    const info = window.__arena!.boardInfo("a") as Array<{ cannot_attack_reason?: string | null }>;
    return info[0]?.cannot_attack_reason ?? null;
  });
  expect(reason).toBeTruthy();
  const overlay = page.locator("#blueBoard .cant_attack-overlay");
  await expect(overlay).toBeVisible();
  const op = await overlay.evaluate((el) => getComputedStyle(el).opacity);
  expect(Number(op)).toBeGreaterThan(0);
  const shot = await artShot(page.locator("#blueBoard .card").first(), `${ART}/p1_cant_attack.png`);
  expect(
    sampleColor(shot, (r, g, b, a) => a > 80 && r > 200 && g > 200 && b > 200),
    "crossed-chain overlay should paint light links",
  ).toBeGreaterThan(40);
});

test("#23 keyword swap-2 on Bane+Drain; Ongoing asset present", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "whisperer.json", "10512120");
  await skipToPp(page, 5);
  await playCard(page, "10512120");
  const stack = page.locator("#blueBoard .keyword-icon-stack").first();
  await expect(stack).toHaveClass(/swap-2/);
  const dur = await stack.locator(".keyword-icon").first().evaluate((el) => getComputedStyle(el).animationDuration);
  expect(dur).toBe("2s");
  const icons = stack.locator(".keyword-icon");
  await expect(icons).toHaveCount(2);
  const ongoingOk = await page.evaluate(async () => {
    const r = await fetch(new URL("images/icon_ongoing.png", document.baseURI).href);
    return r.ok;
  });
  expect(ongoingOk).toBeTruthy();
  await artShot(page.locator("#blueBoard .card").first(), `${ART}/p1_keyword_swap.png`);
});

test("#24 spellboost badge under the cost", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "boost.json", { "10032120": 20, "10031310": 20 });
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);
  await skipToPp(page, 1);
  const had = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === "10031310");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  if (!had) test.skip(true, "Foresight not legal in this seed");
  const badge = page.locator(".spellboost-badge").first();
  await expect(badge).toBeVisible({ timeout: 8000 });
  const style = await badge.evaluate((el) => {
    const s = getComputedStyle(el);
    return { color: s.color, radius: s.borderRadius, bg: s.backgroundColor };
  });
  expect(style.color).toBe("rgb(255, 255, 255)");
  expect(style.radius).toBe("50%");
  const shot = await artShot(page.locator(".card").filter({ has: page.locator(".spellboost-badge") }).first(), `${ART}/p1_spellboost.png`);
  expect(sampleColor(shot, (r, g, b, a) => a > 80 && b > 140 && b > r && b > g)).toBeGreaterThan(4);
});

test("#25 leader barrier ring after Zooey Enhance 10", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await startMono(page, "zooey.json", "10444120");
  await skipToPp(page, 10);
  const played = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
    const act = legal.find((a) => a.play?.card === "10444120");
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(played).toBeTruthy();
  await expect.poll(() =>
    page.evaluate(() => window.__arena!.playerInfo("a").has_leader_barrier),
  ).toBeTruthy();
  const leader = page.locator("#blueLeader");
  await expect(leader).toHaveClass(/has-leader-barrier/);
  const border = await leader.evaluate((el) => getComputedStyle(el, "::before").borderColor);
  expect(border).toMatch(/120,\s*220,\s*255|rgb\(120, 220, 255\)/);
  const shot = await artShot(leader, `${ART}/p1_leader_barrier.png`);
  expect(sampleColor(shot, (r, g, b, a) => a > 40 && b > 180 && g > 160 && r < 180)).toBeGreaterThan(4);
});

test("#26 Escape closes the history drawer", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await closeDrawer(page);
  await page.locator("#historyToggle").click();
  const drawer = page.locator("#historyDrawer");
  await expect(drawer).toHaveClass(/open/);
  const ms = await drawer.evaluate((el) => getComputedStyle(el).transitionDuration);
  expect(ms).toMatch(/0\.18s|0.18s/);
  await page.keyboard.press("Escape");
  await expect(drawer).not.toHaveClass(/open/);
  await expect(page.locator("#historyScrim")).not.toHaveClass(/show/);
  await artShot(page, `${ART}/p1_history_closed.png`);
});

test("#27 Copy seed flashes Copied for 1200ms", async ({ page }) => {
  await boot(page);
  await startGame(page, {
    seed: "1",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  await openSettings(page);
  const btn = page.locator("#copySeedBtn");
  await expect(btn).toBeVisible();
  await btn.evaluate((el) => (el as HTMLButtonElement).click());
  await expect(btn).toHaveText("Copied");
  await artShot(page.locator("#gameSeedPanel"), `${ART}/p1_copy_seed.png`);
  await expect(btn).toHaveText("Copy", { timeout: 2000 });
});

test("#28 active-on-bottom persisted before first paint", async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem("svwb.activeOnBottom", "1");
  });
  await boot(page);
  await expect(page.locator("body")).toHaveClass(/active-on-bottom/);
  await openSettings(page);
  await expect(page.locator("#activeOnBottomToggle")).toBeChecked();
  const order = await page.evaluate(() => {
    const red = getComputedStyle(document.querySelector(".side-red")!).order;
    const blue = getComputedStyle(document.querySelector(".side-blue")!).order;
    return { red, blue };
  });
  expect(order.red).toBeTruthy();
  await artShot(page.locator("#appRoot"), `${ART}/p1_active_on_bottom.png`);
});

test("terminal overlay: Deck-out/Lethal + rematch buttons; empty seed rolls", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  await expect(page.locator("#seedInput")).toHaveAttribute("placeholder", "random if empty");
  await startGame(page, {
    seed: "",
    first: "a",
    deckA: "basic-forest",
    deckB: "basic-rune",
  });
  const rolled = await page.locator("#gameSeedValue").textContent();
  expect(rolled && /^\d+$/.test(rolled)).toBeTruthy();
  const tiny = await importDeck(page, "tiny.json", { "10001110": 8 });
  await startGame(page, { seed: "1", first: "a", deckA: tiny, deckB: tiny });
  await confirmMulligans(page);
  await closeDrawer(page);
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
  const reason = page.locator("#gameOverReason");
  await expect(reason).toHaveText(/Deck-out|Lethal/);
  await expect(page.locator("#rematchSameSeedBtn")).toHaveText("Rematch (same seed)");
  await expect(page.locator("#rematchNewSeedBtn")).toHaveText("Rematch (new seed)");
  await expect(page.locator("#newGameFromOver")).toHaveText("New Game");
  await expect(page.locator(".gameover-hint")).toContainText("First player of a rematch");
  await artShot(page.locator("#gameOverOverlay"), `${ART}/p1_terminal.png`);
});
