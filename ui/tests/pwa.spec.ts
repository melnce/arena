import { expect, test, type Page } from "@playwright/test";

async function boot(page: Page) {
  await page.goto("/");
  await expect(page.locator("#bundleMeta")).toContainText("cards", { timeout: 30_000 });
}

async function assertShellFits(page: Page) {
  const metrics = await page.evaluate(() => {
    const shell = document.getElementById("gameShell");
    const rail = document.getElementById("turnControls");
    const pad = getComputedStyle(shell ?? document.body);
    return {
      scrollW: document.documentElement.scrollWidth,
      innerW: window.innerWidth,
      innerH: window.innerHeight,
      shellH: shell?.getBoundingClientRect().height ?? 0,
      rail: rail?.getBoundingClientRect() ?? { right: 0, bottom: 0, left: 0, top: 0 },
      safeTop: pad.paddingTop,
      safeRight: pad.paddingRight,
      safeBottom: pad.paddingBottom,
      safeLeft: pad.paddingLeft,
    };
  });
  expect(metrics.scrollW).toBeLessThanOrEqual(metrics.innerW + 1);
  expect(metrics.shellH).toBeGreaterThan(metrics.innerH * 0.95);
  expect(metrics.rail.right).toBeLessThanOrEqual(metrics.innerW + 1);
  expect(metrics.rail.bottom).toBeLessThanOrEqual(metrics.innerH + 1);
  expect(metrics.rail.left).toBeGreaterThanOrEqual(-1);
  expect(metrics.rail.top).toBeGreaterThanOrEqual(-1);
  expect(metrics.safeTop).toMatch(/px$/);
  expect(metrics.safeRight).toMatch(/px$/);
  expect(metrics.safeBottom).toMatch(/px$/);
  expect(metrics.safeLeft).toMatch(/px$/);
}

test.describe("iPad landscape standalone shell", () => {
  test.use({
    viewport: { width: 1180, height: 820 },
    isMobile: true,
    hasTouch: true,
  });

  test("shell fills the viewport; rail is not clipped", async ({ page }) => {
    await boot(page);
    await assertShellFits(page);
    await expect(page.locator("#settingsToggle")).toBeVisible();
    await expect(page.locator("#historyToggle")).toBeVisible();
  });
});

test.describe("Android tablet standalone shell", () => {
  test.use({
    viewport: { width: 1280, height: 800 },
    isMobile: true,
    hasTouch: true,
  });

  test("shell fills the viewport; rail is not clipped", async ({ page }) => {
    await boot(page);
    await assertShellFits(page);
    await expect(page.locator("#settingsToggle")).toBeVisible();
    await expect(page.locator("#historyToggle")).toBeVisible();
  });
});

test("manifest, icons, and iOS standalone meta", async ({ page, request }) => {
  await boot(page);
  await expect(page.locator('link[rel="manifest"]')).toHaveAttribute(
    "href",
    "/manifest.webmanifest",
  );
  await expect(page.locator('meta[name="apple-mobile-web-app-capable"]')).toHaveAttribute(
    "content",
    "yes",
  );
  await expect(
    page.locator('meta[name="apple-mobile-web-app-status-bar-style"]'),
  ).toHaveAttribute("content", "black-translucent");
  await expect(page.locator('meta[name="apple-mobile-web-app-title"]')).toHaveAttribute(
    "content",
    "Arena",
  );
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute("content", "#0b0c0f");
  await expect(page.locator('link[rel="apple-touch-icon"]')).toHaveAttribute(
    "href",
    "/icons/apple-touch-icon-180.png",
  );

  const res = await request.get("/manifest.webmanifest");
  expect(res.status()).toBe(200);
  const ctype = res.headers()["content-type"] ?? "";
  expect(ctype).toMatch(/json|manifest/);
  const manifest = (await res.json()) as {
    name: string;
    short_name: string;
    start_url: string;
    scope: string;
    display: string;
    background_color: string;
    theme_color: string;
    icons: Array<{ src: string; sizes: string; purpose?: string }>;
  };
  expect(manifest.name).toBe("Arena");
  expect(manifest.short_name).toBe("Arena");
  expect(manifest.start_url).toBe("/");
  expect(manifest.scope).toBe("/");
  expect(manifest.display).toBe("standalone");
  expect(manifest.background_color).toBe("#0b0c0f");
  expect(manifest.theme_color).toBe("#0b0c0f");
  expect(manifest).not.toHaveProperty("orientation");
  const sizes = manifest.icons.map((i) => i.sizes);
  expect(sizes).toContain("192x192");
  expect(sizes).toContain("512x512");
  expect(manifest.icons.some((i) => i.purpose === "maskable")).toBeTruthy();
  for (const icon of manifest.icons) {
    const ir = await request.get(icon.src);
    expect(ir.status(), icon.src).toBe(200);
  }
  const apple = await request.get("/icons/apple-touch-icon-180.png");
  expect(apple.status()).toBe(200);
});
