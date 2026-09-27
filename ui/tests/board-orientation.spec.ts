import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings, waitEnterAnimation } from "./helpers.ts";

const CRYSTAL = "10631110";
const DECK = { [CRYSTAL]: 40 };

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

async function endTurnApply(page: Page) {
  const ok = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ end_turn?: unknown }>;
    const act = legal.find((a) => a.end_turn);
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

type BoardIds = "red" | "blue";

async function playFollowerRecord(
  page: Page,
  entryUids: Record<BoardIds, string[]>,
): Promise<void> {
  const acting = await page.locator("#turnCounter").getAttribute("data-acting");
  const board: BoardIds = acting === "a" ? "blue" : "red";
  const before = await page.locator(`#${board}Board .card`).count();
  await playCard(page, CRYSTAL);
  const card = page.locator(`#${board}Board .card`).nth(before);
  await expect(card).toBeVisible({ timeout: 5000 });
  await waitEnterAnimation(card);
  const uid = await card.getAttribute("data-uid");
  expect(uid).toBeTruthy();
  entryUids[board].push(uid!);
}

async function fillBoards(page: Page, each: number): Promise<Record<BoardIds, string[]>> {
  const entryUids: Record<BoardIds, string[]> = { red: [], blue: [] };
  for (let round = 0; round < each * 2; round++) {
    for (const side of ["a", "b"] as const) {
      const acting = await page.locator("#turnCounter").getAttribute("data-acting");
      if (acting !== side) {
        await endTurnApply(page);
      }
      const board: BoardIds = side === "a" ? "blue" : "red";
      if (entryUids[board].length < each) {
        await playFollowerRecord(page, entryUids);
      }
    }
  }
  for (const board of ["red", "blue"] as const) {
    await expect(page.locator(`#${board}Board .card`)).toHaveCount(each, { timeout: 5000 });
  }
  return entryUids;
}

type RowCheck = {
  board: BoardIds;
  slots: number[];
  uids: string[];
  flexDirection: string;
  isTop: boolean;
};

async function readBoardRows(page: Page): Promise<{ top: RowCheck; bottom: RowCheck }> {
  return page.evaluate(() => {
    const boards = {
      red: document.getElementById("redBoard")!,
      blue: document.getElementById("blueBoard")!,
    };
    const redTop = boards.red.getBoundingClientRect().top;
    const blueTop = boards.blue.getBoundingClientRect().top;
    const topId: BoardIds = redTop < blueTop ? "red" : "blue";
    const bottomId: BoardIds = redTop < blueTop ? "blue" : "red";

    function row(boardId: BoardIds, isTop: boolean): RowCheck {
      const el = boards[boardId];
      const cards = [...el.querySelectorAll<HTMLElement>(".card")]
        .map((c) => ({
          slot: Number(c.dataset.slot),
          uid: c.dataset.uid ?? "",
          left: c.getBoundingClientRect().left,
        }))
        .sort((a, b) => a.left - b.left);
      return {
        board: boardId,
        slots: cards.map((c) => c.slot),
        uids: cards.map((c) => c.uid),
        flexDirection: getComputedStyle(el).flexDirection,
        isTop,
      };
    }

    return { top: row(topId, true), bottom: row(bottomId, false) };
  });
}

function assertRowOrientation(
  row: RowCheck,
  entryUids: string[],
  position: "top" | "bottom",
): void {
  const n = entryUids.length;
  if (position === "bottom") {
    expect(row.slots).toEqual([...Array(n).keys()]);
    expect(row.uids).toEqual(entryUids);
    expect(row.flexDirection).toBe("row");
  } else {
    expect(row.slots).toEqual([...Array(n).keys()].reverse());
    expect(row.uids).toEqual([...entryUids].reverse());
    expect(row.flexDirection).toBe("row-reverse");
  }
}

async function assertBoardOrientation(
  page: Page,
  entryUids: Record<BoardIds, string[]>,
): Promise<void> {
  const { top, bottom } = await readBoardRows(page);
  assertRowOrientation(top, entryUids[top.board], "top");
  assertRowOrientation(bottom, entryUids[bottom.board], "bottom");
}

async function rightmostCard(page: Page, board: BoardIds) {
  const cards = page.locator(`#${board}Board .card`);
  const n = await cards.count();
  let best = 0;
  let bestLeft = -Infinity;
  for (let i = 0; i < n; i++) {
    const box = await cards.nth(i).boundingBox();
    if (box && box.x > bestLeft) {
      bestLeft = box.x;
      best = i;
    }
  }
  return cards.nth(best);
}

test("board rows face each player — slot 0 at own left", async ({ page }) => {
  test.setTimeout(180_000);
  await boot(page);
  const id = await importDeck(page, "crystalspawn.json", DECK);
  await startGame(page, { seed: "1", first: "a", deckA: id, deckB: id });
  await confirmMulligans(page);
  await closeDrawer(page);

  const entryUids = await fillBoards(page, 3);
  await assertBoardOrientation(page, entryUids);
  await artShot(page.locator("#appRoot"), `${ART}/board_orientation_default.png`);

  // A's turn so the bottom row (blue) can attack.
  await endTurnApply(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 5000 });

  // Attack: bottom attacker removes top row's first-entered follower (slot 0, rightmost on top).
  const { top, bottom } = await readBoardRows(page);
  const topFirst = await rightmostCard(page, top.board);
  expect(await topFirst.getAttribute("data-slot")).toBe("0");

  const attackerBoard = bottom.board;
  const targetBoard = top.board;
  const attacker = page
    .locator(`#${attackerBoard}Board .card.can-attack, #${attackerBoard}Board .card.legal-attack`)
    .first();
  await expect(attacker).toBeVisible({ timeout: 10_000 });
  const targetSlot = await topFirst.getAttribute("data-slot");
  const targetUid = await topFirst.getAttribute("data-uid");
  expect(targetSlot).toBeTruthy();
  expect(targetUid).toBeTruthy();

  await attacker.click();
  await topFirst.click();

  const applied = await page.evaluate(() => {
    const actions = window.__arena!.actions() as Array<{
      attack?: { target: { slot?: number } | "leader"; attacker_slot: number };
    }>;
    const last = actions[actions.length - 1];
    return last?.attack ?? null;
  });
  expect(applied).toBeTruthy();
  expect(applied!.target).toEqual({ slot: Number(targetSlot) });

  await expect(page.locator(`#${targetBoard}Board .card[data-uid="${targetUid}"]`)).toHaveCount(0, {
    timeout: 5000,
  });
  entryUids[targetBoard] = entryUids[targetBoard].filter((u) => u !== targetUid);
  await assertBoardOrientation(page, entryUids);

  // Targeting still keys on data-slot: top row rightmost remains slot 0 when 2+ remain.
  if (entryUids[targetBoard].length >= 2) {
    const topSlot0 = await rightmostCard(page, targetBoard);
    expect(await topSlot0.getAttribute("data-slot")).toBe("0");
  }

  // Active-on-bottom + B acting: red bottom, blue top.
  await openSettings(page);
  await page.locator("#activeOnBottomToggle").check();
  await closeDrawer(page);
  await endTurnApply(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "b", { timeout: 5000 });
  await assertBoardOrientation(page, entryUids);
  await artShot(page.locator("#appRoot"), `${ART}/board_orientation_flipped.png`);

  // Toggle still on, A acting: red top again.
  await endTurnApply(page);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-acting", "a", { timeout: 5000 });
  await assertBoardOrientation(page, entryUids);
});
