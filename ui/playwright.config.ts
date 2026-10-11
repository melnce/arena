import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 90_000,
  fullyParallel: false,
  retries: 0,
  projects: [
    {
      name: "chromium",
      testIgnore: /bot-pool\.spec\.ts$/,
    },
    {
      name: "chromium-bot-pool",
      testMatch: /bot-pool\.spec\.ts$/,
      timeout: 360_000,
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
