import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  reporter: "list",
  timeout: 90_000,
  fullyParallel: false,
  workers: process.env.CI ? 1 : undefined,
  retries: 0,
  projects: [
    {
      name: "chromium",
      testIgnore: /bot-pool(-kill-done)?\.spec\.ts$/,
    },
    {
      name: "chromium-bot-pool",
      testMatch: /bot-pool\.spec\.ts$/,
      timeout: 360_000,
    },
    {
      name: "chromium-bot-pool-kill-done",
      testMatch: /bot-pool-kill-done\.spec\.ts$/,
      timeout: 720_000,
    },
  ],
  use: {
    baseURL: "http://127.0.0.1:4173",
    browserName: "chromium",
    headless: true,
  },
  webServer: {
    command: "npm run preview -- --host 127.0.0.1 --port 4173",
    port: 4173,
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
