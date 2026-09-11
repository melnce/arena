import { expect, type Locator, type Page } from "@playwright/test";
import { inflateSync } from "node:zlib";
import { readFileSync } from "node:fs";

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
  const color = await wrapperOutline(card);
  if (kind === "none") {
    expect(color === "none" || color === "rgba(0, 0, 0, 0)" || color === "transparent").toBeTruthy();
    return;
  }
  expect(color).toBe(kind === "yellow" ? YELLOW : GREEN);
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
  await page.waitForTimeout(250);
  let prev: { red: number; blue: number } | null = null;
  for (let i = 0; i < 20; i++) {
    const m = await page.evaluate(() => {
      const red = document.querySelector("#redHand .card")?.getBoundingClientRect().height ?? 0;
      const blue = document.querySelector("#blueHand .card")?.getBoundingClientRect().height ?? 0;
      return { red, blue };
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
