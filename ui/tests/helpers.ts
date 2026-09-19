import { expect, type Locator, type Page } from "@playwright/test";
import { inflateSync } from "node:zlib";
import { readFileSync } from "node:fs";
import { copyFile, mkdir as mkdirP } from "node:fs/promises";

export const ART = "/opt/cursor/artifacts";

const TRANSIENT_ANIMS = [
  "card-enter",
  "barrier-flash-kf",
  "floating-combat-card-flash",
  "stat-flash",
];

/** Wait until a `.card-enter` animation has finished (class may remain). */
export async function waitEnterAnimation(card: Locator): Promise<void> {
  await waitAnimationEnd(card, "card-enter");
}

/** Wait for a finite CSS animation to finish. Looping or already-done animations return immediately. */
export async function waitAnimationEnd(el: Locator, name?: string): Promise<void> {
  await el.evaluate(async (node, animName) => {
    const active = (node.getAnimations?.() ?? []).filter((a) => {
      if (a.constructor.name !== "CSSAnimation") return false;
      const n = (a as unknown as { animationName?: string }).animationName ?? "";
      if (animName && !n.includes(animName)) return false;
      return a.playState === "running" || a.playState === "pending";
    });
    if (active.length === 0) return;
    const infinite = active.some((a) => {
      const effect = a.effect as { getTiming?: () => { iterations?: number } } | null;
      return effect?.getTiming?.().iterations === Infinity;
    });
    if (infinite) return;
    await Promise.race([
      Promise.all(active.map((a) => a.finished.catch(() => undefined))),
      new Promise<void>((resolve) => window.setTimeout(resolve, 800)),
    ]);
  }, name);
}

/** Wait for a CSS transition on `property` to finish, or return if none is running. */
export async function waitTransitionEnd(el: Locator, property?: string): Promise<void> {
  await el.evaluate(async (node, prop) => {
    const running = (node.getAnimations?.() ?? []).some((a) => {
      if (a.constructor.name !== "CSSTransition") return false;
      const t = a as unknown as { transitionProperty?: string };
      return !prop || t.transitionProperty === prop;
    });
    if (!running) return;
    const style = getComputedStyle(node);
    const durs = style.transitionDuration.split(",").map((d) => {
      const n = parseFloat(d);
      return d.includes("ms") ? n : n * 1000;
    });
    const maxMs = Math.max(80, ...durs) + 80;
    await Promise.race([
      new Promise<void>((resolve) => {
        const onEnd = (e: TransitionEvent) => {
          if (!prop || e.propertyName === prop) {
            node.removeEventListener("transitionend", onEnd);
            resolve();
          }
        };
        node.addEventListener("transitionend", onEnd);
      }),
      new Promise<void>((resolve) => window.setTimeout(resolve, maxMs)),
    ]);
  }, property);
}

/**
 * Wait out one-shot card animations (enter / barrier-flash / FCT flash).
 * Does not wait for looping pulses (playPulse, barrier-pulse, keyword swap).
 */
export async function waitTransientCardAnimations(card: Locator): Promise<void> {
  await card.evaluate(async (el, transients: string[]) => {
    const active = () =>
      (el.getAnimations?.() ?? []).filter((a) => {
        if (a.constructor.name !== "CSSAnimation") return false;
        const n = (a as unknown as { animationName?: string }).animationName ?? "";
        if (!transients.some((t) => n.includes(t))) return false;
        return a.playState === "running" || a.playState === "pending";
      });
    const now = active();
    if (now.length === 0) return;
    await Promise.race([
      Promise.all(now.map((a) => a.finished.catch(() => undefined))),
      new Promise<void>((resolve) => window.setTimeout(resolve, 800)),
    ]);
  }, TRANSIENT_ANIMS);
}

/**
 * Open the settings drawer and wait for its 0.18s transform to settle.
 * Retries the toggle click if a reload/bind race swallows the first one
 * (`toPass` re-evaluates one assertion — not a test-level retry).
 */
export async function openSettings(page: Page): Promise<void> {
  const drawer = page.locator("#settingsDrawer");
  const toggle = page.locator("#settingsToggle");
  await expect(toggle).toBeVisible();
  await expect(async () => {
    const already = await drawer.evaluate((el) => el.classList.contains("open"));
    if (!already) await toggle.click();
    await expect(drawer).toHaveClass(/open/, { timeout: 1500 });
  }).toPass({ timeout: 10_000 });
  await waitTransitionEnd(drawer, "transform");
}

/** Hover a history row and wait for the preview `<img>` to load or error. */
export async function waitHistoryPreview(
  row: Locator,
  preview: Locator,
): Promise<"load" | "error"> {
  await row.hover();
  await expect(preview).toBeVisible();
  const img = preview.locator("img");
  await expect(img).toHaveCount(1);
  return img.evaluate((el) => {
    const node = el as HTMLImageElement;
    if (node.complete) return node.naturalWidth > 0 ? "load" : "error";
    return new Promise<"load" | "error">((resolve) => {
      node.addEventListener("load", () => resolve("load"), { once: true });
      node.addEventListener("error", () => resolve("error"), { once: true });
    });
  });
}

/** Write to /tmp first — /opt/cursor/artifacts close() can EIO and must not fail a test. */
export async function artShot(
  target: { screenshot: (opts: { path: string; fullPage?: boolean }) => Promise<Buffer> },
  path: string,
  opts: { fullPage?: boolean } = {},
): Promise<string> {
  const fallback = `/tmp/${path.split("/").pop()}`;
  await target.screenshot({ path: fallback, ...opts });
  try {
    await mkdirP(ART, { recursive: true });
    await copyFile(fallback, path);
    return path;
  } catch {
    return fallback;
  }
}

export const YELLOW = "rgb(255, 212, 0)";
export const GREEN = "rgb(57, 217, 138)";

export async function wrapperOutline(card: Locator): Promise<string> {
  return card.locator(".card-image-wrapper").evaluate((el) => {
    const s = getComputedStyle(el);
    if (s.outlineStyle === "none" || s.outlineWidth === "0px") return "none";
    return s.outlineColor;
  });
}

export async function assertGlow(card: Locator, kind: "yellow" | "green" | "none"): Promise<void> {
  if (kind === "none") {
    await expect
      .poll(async () => {
        const color = await wrapperOutline(card);
        return color === "none" || color === "rgba(0, 0, 0, 0)" || color === "transparent";
      })
      .toBeTruthy();
    return;
  }
  await expect.poll(() => wrapperOutline(card)).toBe(kind === "yellow" ? YELLOW : GREEN);
}

export function sampleColor(
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

/** Screenshot + count matching pixels, re-trying until the sample holds. */
export async function expectPngHits(
  target: { screenshot: (opts: { path: string; fullPage?: boolean }) => Promise<Buffer> },
  path: string,
  pred: (r: number, g: number, b: number, a: number) => boolean,
  min: number,
  label: string,
): Promise<void> {
  await expect(async () => {
    const shot = await artShot(target, path);
    expect(sampleColor(shot, pred), label).toBeGreaterThan(min);
  }).toPass({ timeout: 8_000 });
}

function paeth(a: number, b: number, c: number): number {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  if (pa <= pb && pa <= pc) return a;
  if (pb <= pc) return b;
  return c;
}

export function pngRgba(buf: Buffer): { width: number; height: number; data: Buffer } {
  if (buf.toString("ascii", 1, 4) !== "PNG") throw new Error("not a png");
  let off = 8;
  let width = 0;
  let height = 0;
  let colorType = 0;
  const idats: Buffer[] = [];
  while (off + 8 <= buf.length) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString("ascii", off + 4, off + 8);
    const data = buf.subarray(off + 8, off + 8 + len);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      colorType = data[9];
    } else if (type === "IDAT") {
      idats.push(Buffer.from(data));
    } else if (type === "IEND") {
      break;
    }
    off += 12 + len;
  }
  const bpp = colorType === 6 ? 4 : colorType === 2 ? 3 : 0;
  if (!bpp) throw new Error(`unsupported png colorType ${colorType}`);
  const raw = inflateSync(Buffer.concat(idats));
  const stride = width * bpp;
  const out = Buffer.alloc(height * width * 4);
  let prev = Buffer.alloc(stride);
  let src = 0;
  for (let y = 0; y < height; y++) {
    const filter = raw[src++];
    const row = Buffer.alloc(stride);
    for (let i = 0; i < stride; i++) {
      const x = raw[src++];
      const a = i >= bpp ? row[i - bpp] : 0;
      const b = prev[i];
      const c = i >= bpp ? prev[i - bpp] : 0;
      let v = x;
      if (filter === 1) v = (x + a) & 255;
      else if (filter === 2) v = (x + b) & 255;
      else if (filter === 3) v = (x + Math.floor((a + b) / 2)) & 255;
      else if (filter === 4) v = (x + paeth(a, b, c)) & 255;
      row[i] = v;
    }
    for (let x = 0; x < width; x++) {
      const si = x * bpp;
      const di = (y * width + x) * 4;
      out[di] = row[si];
      out[di + 1] = row[si + 1];
      out[di + 2] = row[si + 2];
      out[di + 3] = bpp === 4 ? row[si + 3] : 255;
    }
    prev = row;
  }
  return { width, height, data: out };
}

const SUCCESS_GREEN = { r: 57, g: 217, b: 138 };
const WARN_YELLOW = { r: 255, g: 212, b: 0 };

function nearRgb(
  r: number,
  g: number,
  b: number,
  ref: { r: number; g: number; b: number },
  tol = 35,
): boolean {
  return Math.abs(r - ref.r) <= tol && Math.abs(g - ref.g) <= tol && Math.abs(b - ref.b) <= tol;
}

/** Mid-side edge band only — skips cost / ATK / DEF / E·SE badge corners. */
export function assertPngExclusiveRing(path: string, kind: "yellow" | "green"): void {
  const img = pngRgba(readFileSync(path));
  const band = Math.max(6, Math.min(12, Math.floor(img.width * 0.08)));
  const skipX = Math.floor(img.width * 0.28);
  const skipY = Math.floor(img.height * 0.28);
  let foreign = 0;
  let expected = 0;
  const visit = (x: number, y: number) => {
    const i = (y * img.width + x) * 4;
    if (img.data[i + 3] < 80) return;
    const r = img.data[i];
    const g = img.data[i + 1];
    const b = img.data[i + 2];
    const isGreen = nearRgb(r, g, b, SUCCESS_GREEN);
    const isYellow = nearRgb(r, g, b, WARN_YELLOW);
    if (kind === "yellow") {
      if (isGreen) foreign += 1;
      if (isYellow) expected += 1;
    } else {
      if (isYellow) foreign += 1;
      if (isGreen) expected += 1;
    }
  };
  // Left mid-side + bottom mid — skips cost, ATK/DEF, and the E/SE badge.
  for (let y = skipY; y < img.height - skipY; y++) {
    for (let x = 0; x < band; x++) visit(x, y);
  }
  for (let x = skipX; x < img.width - skipX; x++) {
    for (let y = img.height - band; y < img.height; y++) visit(x, y);
  }
  expect(expected, `${path} edge should contain ${kind}`).toBeGreaterThan(4);
  expect(foreign, `${path} edge must not mix the other ring colour`).toBe(0);
}

export async function assertOuterCardNotDashedGreen(card: Locator): Promise<void> {
  const outer = await card.evaluate((el) => {
    const s = getComputedStyle(el);
    return { style: s.outlineStyle, color: s.outlineColor, width: s.outlineWidth };
  });
  const dashedGreen =
    outer.style === "dashed" &&
    /57,\s*217,\s*138/.test(outer.color) &&
    outer.width !== "0px";
  expect(dashedGreen, "outer .card must not carry the idle dashed green ring").toBe(false);
}

export function assertPngLeftEdge(path: string, kind: "yellow" | "green"): void {
  const img = pngRgba(readFileSync(path));
  const y0 = Math.floor(img.height * 0.35);
  const y1 = Math.floor(img.height * 0.65);
  let hits = 0;
  for (let y = y0; y < y1; y++) {
    for (let x = 0; x < Math.min(6, img.width); x++) {
      const i = (y * img.width + x) * 4;
      const r = img.data[i];
      const g = img.data[i + 1];
      const b = img.data[i + 2];
      const a = img.data[i + 3];
      if (a < 80) continue;
      if (kind === "yellow" && r > 200 && g > 170 && b < 90) hits += 1;
      if (kind === "green" && g > 160 && r < 140 && b < 200) hits += 1;
    }
  }
  expect(hits, `${path} left edge should be ${kind}`).toBeGreaterThan(4);
}

export async function waitCardSizeStable(page: Page): Promise<void> {
  const red = page.locator("#redHand .card").first();
  const blue = page.locator("#blueHand .card").first();
  if (await red.count()) await waitEnterAnimation(red);
  if (await blue.count()) await waitEnterAnimation(blue);
  let prev: { red: number; blue: number } | null = null;
  for (let i = 0; i < 20; i++) {
    const m = await page.evaluate(() => {
      const redH = document.querySelector("#redHand .card")?.getBoundingClientRect().height ?? 0;
      const blueH = document.querySelector("#blueHand .card")?.getBoundingClientRect().height ?? 0;
      return { red: redH, blue: blueH };
    });
    if (
      prev &&
      Math.abs(prev.red - m.red) < 0.25 &&
      Math.abs(prev.blue - m.blue) < 0.25
    ) {
      return;
    }
    prev = m;
    await page.waitForTimeout(50);
  }
}

export async function assertPromptClearsCards(page: Page): Promise<void> {
  const bar = page.locator(".choice-prompt-bar");
  await expect(bar).toBeVisible();
  const overlap = await page.evaluate(() => {
    const barEl = document.querySelector(".choice-prompt-bar");
    if (!barEl) return "no-bar";
    const br = barEl.getBoundingClientRect();
    const hits: string[] = [];
    for (const el of document.querySelectorAll<HTMLElement>(".card, .leader-attack-strip")) {
      const r = el.getBoundingClientRect();
      const inter = !(
        br.right < r.left ||
        br.left > r.right ||
        br.bottom < r.top ||
        br.top > r.bottom
      );
      if (inter) hits.push(el.id || el.className);
    }
    return hits.join(",") || "";
  });
  expect(overlap, "prompt bar must not cover a card or leader bar").toBe("");
}
