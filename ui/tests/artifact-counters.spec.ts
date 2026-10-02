import { expect, test, type Page } from "@playwright/test";
import { ART, artShot, openSettings } from "./helpers.ts";

const ANALYZING = "90071130";
const ANCIENT = "90071140";
const MYSTIC = "90071150";
const BEAT_BREAKER = "10771120";
const MYUU = "10774120";
const SCARLET = "10774110";
const WARP_SLASH = "10773310";

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
  await page.locator("#seedInput").fill("42");
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

async function closeDrawer(page: Page) {
  await page.evaluate(() => {
    document.getElementById("settingsDrawer")?.classList.remove("open");
    document.getElementById("settingsScrim")?.classList.remove("show");
  });
}

async function endTurn(page: Page) {
  await page.evaluate(() => {
    const acting = document.getElementById("turnCounter")?.dataset.acting;
    const id = acting === "a" ? "endTurnBlue" : "endTurnRed";
    (document.getElementById(id) as HTMLButtonElement | null)?.click();
  });
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

const ARTIFACT_TOKEN_IDS = [ANALYZING, ANCIENT, MYSTIC];

async function artifactDistinctCount(page: Page): Promise<number> {
  return page.evaluate((tokenIds) => {
    const full = window.__arena!.full() as {
      players: { a: { enter_counts: Record<string, number> } };
    };
    const counts = full.players.a.enter_counts ?? {};
    return tokenIds.filter((id) => (counts[id] ?? 0) > 0).length;
  }, ARTIFACT_TOKEN_IDS);
}

async function playArtifactsUntil(page: Page, target: number) {
  for (let i = 0; i < 80; i++) {
    const k = await artifactDistinctCount(page);
    if (k >= target) return k;
    const snap = await page.evaluate((tokenIds) => {
      const full = window.__arena!.full() as {
        active: string;
        players: { a: { pp: number; hand: Array<{ card: string }> } };
      };
      const legal = window.__arena!.legal() as Array<{ play?: { card: string } }>;
      const arts = tokenIds.filter((id) => legal.some((a) => a.play?.card === id));
      const anyPlay = legal.find((a) => a.play);
      return {
        active: full.active,
        arts,
        anyPlay: anyPlay?.play?.card ?? null,
      };
    }, ARTIFACT_TOKEN_IDS);
    if (snap.active === "a" && snap.arts.length > 0) {
      await playCard(page, snap.arts[0]!);
      continue;
    }
    if (snap.active === "a" && snap.anyPlay) {
      await playCard(page, snap.anyPlay);
      continue;
    }
    await endTurn(page);
  }
  return artifactDistinctCount(page);
}

test("artifact counters in tooltips track engine enter_counts", async ({ page }) => {
  test.setTimeout(120_000);
  await boot(page);
  const deck = await importDeck(page, "portal-artifacts.json", {
    [ANALYZING]: 12,
    [ANCIENT]: 12,
    [MYSTIC]: 12,
    [BEAT_BREAKER]: 4,
    [MYUU]: 4,
    [SCARLET]: 4,
    [WARP_SLASH]: 4,
  });
  await startGame(page, deck);
  await confirmMulligans(page);
  await closeDrawer(page);

  const xCard = await page.evaluate((ids) => {
    const hand = (window.__arena!.full() as { players: { a: { hand: Array<{ card: string }> } } })
      .players.a.hand;
    return ids.find((id) => hand.some((c) => c.card === id)) ?? null;
  }, [SCARLET, WARP_SLASH]);
  if (xCard) {
    const card = page.locator(`#blueHand .card[data-card='${xCard}']`).first();
    await card.hover();
    await expect(page.locator("#cardTooltip")).toContainText("Artifacts 0");
    await artShot(card, `${ART}/artifact_x_at_zero.png`);
  }

  const k = await playArtifactsUntil(page, 3);
  expect(k).toBeGreaterThanOrEqual(3);

  if (xCard) {
    const card = page.locator(`#blueHand .card[data-card='${xCard}']`).first();
    await card.hover();
    await expect(page.locator("#cardTooltip")).toContainText(`Artifacts ${k}`);
    await artShot(card, `${ART}/artifact_x_at_${k}.png`);
  }

  const beat = page.locator(`#blueHand .card[data-card='${BEAT_BREAKER}']`).first();
  if (await beat.count()) {
    await beat.hover();
    await expect(page.locator("#cardTooltip")).toContainText(`Artifacts ${k}/3`);
    await expect(page.locator("#cardTooltip .dynamic-counter-line.gate-met")).toContainText(
      `Artifacts ${k}/3`,
    );
    await expect(beat).toHaveClass(/enhance-ready/);
    await artShot(beat, `${ART}/artifact_beat_breaker_met.png`);
  }

  const myuu = page.locator(`#blueHand .card[data-card='${MYUU}']`).first();
  if (await myuu.count()) {
    await myuu.hover();
    await expect(page.locator("#cardTooltip")).toContainText(`Artifacts ${k}/3`);
    await expect(myuu).toHaveClass(/playable-glow/);
    await expect(myuu).not.toHaveClass(/enhance-ready/);
    await artShot(myuu, `${ART}/artifact_myuu_no_yellow.png`);
  }
});
