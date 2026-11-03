import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e", testMatch: "design-direction.spec.ts", workers: 1,
  timeout: 120_000, expect: { timeout: 20_000 },
  outputDir: "../target/ui-design-results", reporter: "list",
  use: { baseURL: process.env.UI_TEST_BASE_URL ?? "http://127.0.0.1:5173", viewport: { width: 1600, height: 1100 } },
});
