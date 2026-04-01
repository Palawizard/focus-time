// noinspection JSUnusedGlobalSymbols
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  use: {
    baseURL: "http://localhost:1420",
    headless: true,
  },
  webServer: {
    command: "corepack pnpm dev -- --host 127.0.0.1",
    port: 1420,
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
