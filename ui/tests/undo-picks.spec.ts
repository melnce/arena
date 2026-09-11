import { expect, test, type Page } from "@playwright/test";

const FIGHTER = "10001110";
const SOUL_TUNING = "10751310";

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

async function startGame(page: Page, deckId: string) {
  await openSettings(page);
  await page.locator("#modeSelect").selectOption("hotseat");
  await page.locator("#seedInput").fill("1");
  await page.locator("#firstSelect").selectOption("a");
  await page.locator("#blueDeckSelect").selectOption(deckId);
  await page.locator("#redDeckSelect").selectOption(deckId);
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

async function chooseOnce(page: Page) {
  const ok = await page.evaluate(() => {
    const legal = window.__arena!.legal() as Array<{ choose?: unknown }>;
    const act = legal.find((a) => "choose" in a);
    if (!act) return false;
    window.__arena!.apply(act);
    return true;
  });
  expect(ok, "expected legal choose").toBeTruthy();
}

type ChoiceSnap = {
  hash: string;
  phase: string;
  remaining: number | null;
  chooseCount: number;
};

async function snap(page: Page): Promise<ChoiceSnap> {
  return page.evaluate(() => {
    const full = window.__arena!.full() as {
      phase: string | { choice?: { node?: { targets?: { remaining?: number } } } };
    };
    const phase = full.phase;
    const remaining =
      typeof phase === "object" && phase.choice?.node?.targets?.remaining != null
        ? phase.choice.node.targets.remaining
        : null;
    const legal = window.__arena!.legal() as Array<{ choose?: unknown }>;
    return {
      hash: window.__arena!.hash(),
      phase: typeof phase === "string" ? phase : "choice",
      remaining,
      chooseCount: legal.filter((a) => "choose" in a).length,
    };
  });
}

test("Soul Tuning: Ctrl+Z / Ctrl+Y undo and redo one pick at a time", async ({ page }) => {
  test.setTimeout(90_000);
  await boot(page);
  const id = await importDeck(page, "soul-tuning.json", { [FIGHTER]: 20, [SOUL_TUNING]: 20 });
  await startGame(page, id);
  await confirmMulligans(page);
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });

  for (let i = 0; i < 24; i++) {
    const st = await page.evaluate((ids) => {
      const full = window.__arena!.full() as {
        active: "a" | "b";
        players: { a: { field: unknown[]; pp: number } };
      };
      const legal = window.__arena!.legal() as Array<{
        play?: { card: string };
        end_turn?: unknown;
      }>;
      const own = full.players.a.field.filter(Boolean).length;
      return {
        active: full.active,
        own,
        canFighter: legal.some((a) => a.play?.card === ids.fighter),
        canTune: legal.some((a) => a.play?.card === ids.tune),
        canEnd: legal.some((a) => a.end_turn),
      };
    }, { fighter: FIGHTER, tune: SOUL_TUNING });
    if (st.active === "a" && st.own >= 2 && st.canTune) break;
    if (st.active === "a" && st.own < 2 && st.canFighter) {
      await playCard(page, FIGHTER);
      continue;
    }
    if (st.canEnd) {
      await applyFirst(page, "end_turn");
      continue;
    }
    throw new Error(`stuck driving to Soul Tuning (i=${i})`);
  }

  const beforePlay = await snap(page);
  expect(beforePlay.phase).toMatch(/main/);
  await playCard(page, SOUL_TUNING);
  await expect(page.locator("#turnCounter")).toHaveAttribute("data-phase", "choice");
  const afterPlay = await snap(page);
  expect(afterPlay.remaining).toBe(2);
  expect(afterPlay.chooseCount).toBeGreaterThan(0);
  await expect(page.locator(".choice-prompt-bar")).toBeVisible();

  await chooseOnce(page);
  const afterPick1 = await snap(page);
  expect(afterPick1.phase).toBe("choice");
  expect(afterPick1.remaining).toBe(1);
  expect(afterPick1.chooseCount).toBeGreaterThan(0);
  await expect(page.locator(".choice-prompt-bar")).toBeVisible();

  await chooseOnce(page);
  const afterPick2 = await snap(page);
  expect(afterPick2.phase).toMatch(/main/);
  expect(afterPick2.hash).not.toBe(afterPick1.hash);

  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());

  await page.keyboard.press("Control+z");
  const z1 = await snap(page);
  expect(z1.hash).toBe(afterPick1.hash);
  expect(z1.phase).toBe("choice");
  expect(z1.remaining).toBe(1);
  expect(z1.chooseCount).toBeGreaterThan(0);
  await expect(page.locator(".choice-prompt-bar")).toBeVisible();

  await page.keyboard.press("Control+z");
  const z2 = await snap(page);
  expect(z2.hash).toBe(afterPlay.hash);
  expect(z2.phase).toBe("choice");
  expect(z2.remaining).toBe(2);
  expect(z2.chooseCount).toBeGreaterThan(0);
  await expect(page.locator(".choice-prompt-bar")).toBeVisible();

  await page.keyboard.press("Control+z");
  const z3 = await snap(page);
  expect(z3.hash).toBe(beforePlay.hash);
  expect(z3.phase).toMatch(/main/);

  await page.keyboard.press("Control+y");
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(afterPlay.hash);
  await page.keyboard.press("Control+y");
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(afterPick1.hash);
  await page.keyboard.press("Control+y");
  expect(await page.evaluate(() => window.__arena!.hash())).toBe(afterPick2.hash);
});
