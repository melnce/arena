import { expect, test, type Page } from "@playwright/test";
import { ART, artShot } from "./helpers.ts";

const TIKOH = "10463110";
const FIGHTER = "10001110";

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
  return `import-${name.replace(/\.json$/i, "")}`;
}

async function startGame(page: Page, deckId: string, deckB?: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckId);
  await page.locator("#redDeckSelect").selectOption(deckB ?? deckId);
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function readyTikoh(page: Page) {
  await boot(page);
  const me = await importDeck(page, "hitpad-tikoh.json", { [TIKOH]: 40 });
  const them = await importDeck(page, "hitpad-opp.json", { [FIGHTER]: 40 });
  await startGame(page, me, them);
  await confirmMulligans(page);
  await closeDrawer(page);
  await playCard(page, TIKOH);
  await applyFirst(page, "end_turn");
  await applyFirst(page, "end_turn");
  const card = page.locator("#blueBoard .card[data-card='10463110']").first();
  await expect(card).toHaveClass(/can-attack/);
  return card;
}

async function hitsAt(
  page: Page,
  x: number,
  y: number,
): Promise<{ id: string; classes: string; leader: boolean; evo: boolean; hp: boolean }> {
  return page.evaluate(({ x: px, y: py }) => {
    const el = document.elementFromPoint(px, py) as HTMLElement | null;
    const host = el?.closest<HTMLElement>(".leader-attack-strip, .evo-btn, .leader-hp-readout");
    const node = host ?? el;
    return {
      id: node?.id ?? "",
      classes: node?.className ?? "",
      leader: !!node?.closest(".leader-attack-strip") || !!node?.id?.endsWith("Leader"),
      evo: !!node?.closest(".evo-btn"),
      hp: !!node?.closest(".leader-hp-readout"),
    };
  }, { x, y });
}

async function stripPoints(page: Page) {
  const box = await page.locator("#redLeader").boundingBox();
  expect(box, "red leader box").toBeTruthy();
  const cx = box!.x + box!.width / 2;
  const cy = box!.y + box!.height / 2;
  return { box: box!, cx, cy, above: cy - 20, below: cy + 20, above15: box!.y - 15 };
}

async function assertPendingHits(page: Page) {
  const card = page.locator("#blueBoard .card[data-card='10463110']").first();
  await card.click();
  await expect(page.locator("#redLeader")).toHaveClass(/attack-drop-hot/);
  const pts = await stripPoints(page);
  for (const y of [pts.cy, pts.above, pts.below]) {
    const hit = await hitsAt(page, pts.cx, y);
    expect(hit.leader, `pending hit at y=${y}`).toBe(true);
  }
  await artShot(page.locator("#redLeader"), `${ART}/leader_hitpad_pending.png`);

  const before = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { field: Array<{ flags: { attacks_left: number } } | null> } };
    };
    return full.players.a.field.find((f) => f)?.flags.attacks_left ?? -1;
  });
  const cb = await card.boundingBox();
  expect(cb).toBeTruthy();
  await page.mouse.move(cb!.x + cb!.width / 2, cb!.y + cb!.height / 2);
  await page.mouse.down();
  await page.mouse.move(pts.cx, pts.above15, { steps: 12 });
  await expect(page.locator("#redLeader")).toHaveClass(/attack-drop-hot/);
  await page.mouse.up();
  const after = await page.evaluate(() => {
    const full = window.__arena!.full() as {
      players: { a: { field: Array<{ flags: { attacks_left: number } } | null> } };
    };
    return full.players.a.field.find((f) => f)?.flags.attacks_left ?? -1;
  });
  expect(after).toBe(0);
  expect(before).toBeGreaterThan(0);
}

async function assertIdleHits(page: Page) {
  const evo = page.locator("#redNormalEvo");
  const hp = page.locator("#redLeaderHp");
  const eb = await evo.boundingBox();
  const hb = await hp.boundingBox();
  expect(eb && hb).toBeTruthy();
  const evoHit = await hitsAt(page, eb!.x + eb!.width / 2, eb!.y + eb!.height / 2);
  const hpHit = await hitsAt(page, hb!.x + hb!.width / 2, hb!.y + hb!.height / 2);
  expect(evoHit.evo).toBe(true);
  expect(hpHit.hp).toBe(true);
}

test("leader hit-pad: pending attack hits the strip and 20px pad; idle hits evo/HP", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await readyTikoh(page);
  await assertIdleHits(page);
  await assertPendingHits(page);
});

test.describe("touch", () => {
  test.use({ hasTouch: true, viewport: { width: 1024, height: 768 } });
  test("leader hit-pad at 1024x768 with hasTouch", async ({ page }) => {
    test.setTimeout(90_000);
    await readyTikoh(page);
    await assertIdleHits(page);
    await assertPendingHits(page);
  });
});
