import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, devices } from "@playwright/test";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const webPort = Number(process.env.E2E_WEB_PORT ?? "5175");
const apiPort = Number(process.env.E2E_API_PORT ?? "19083");
const webBaseURL = process.env.E2E_WEB_BASE_URL ?? `http://127.0.0.1:${webPort}`;
const apiOrigin = process.env.E2E_API_ORIGIN ?? `http://127.0.0.1:${apiPort}`;
const apiBaseURL = process.env.E2E_API_BASE_URL ?? `${apiOrigin}/api/v1`;
const runRoot = process.env.GITY_E2E_ROOT
  ? path.resolve(process.env.GITY_E2E_ROOT)
  : fs.mkdtempSync(path.join(os.tmpdir(), "gity-e2e-"));

const toSlash = (value: string): string => value.replace(/\\/g, "/");
const sqlitePath = toSlash(path.join(runRoot, "gity.db"));
const repoRoot = toSlash(path.join(runRoot, "repos"));
const storageRoot = toSlash(path.join(runRoot, "storage"));
const searchRoot = toSlash(path.join(runRoot, "search-index"));
const binRoot = toSlash(path.join(runRoot, "bin"));

for (const dir of [repoRoot, storageRoot, searchRoot, binRoot]) {
  fs.mkdirSync(dir, { recursive: true });
}

process.env.E2E_WEB_BASE_URL = webBaseURL;
process.env.E2E_API_BASE_URL = apiBaseURL;
const backendEnv = Object.fromEntries(
  Object.entries(process.env).filter(([key, value]) => key !== "GOCACHE" && value !== undefined),
) as Record<string, string>;

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: [
    ["list"],
    ["html", { outputFolder: "../.tmp-e2e/playwright-report", open: "never" }],
  ],
  outputDir: "../.tmp-e2e/playwright-results",
  use: {
    baseURL: webBaseURL,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },
  webServer: [
    {
      command: "node web/e2e/support/start-backend.mjs",
      cwd: rootDir,
      url: `${apiOrigin}/api/health`,
      reuseExistingServer: false,
      timeout: 240_000,
      env: {
        ...backendEnv,
        GITY_APP__ENVIRONMENT: "production",
        GITY_HTTP__ADDRESS: `127.0.0.1:${apiPort}`,
        GITY_HTTP__BASE_URL: apiOrigin,
        GITY_DATABASE__DRIVER: "sqlite",
        GITY_DATABASE__DSN: `file:${sqlitePath}?_pragma=foreign_keys(1)`,
        GITY_DATABASE__NODE_ID: "1",
        GITY_GIT__REPO_ROOT: repoRoot,
        GITY_STORAGE__DRIVER: "local",
        GITY_STORAGE__ROOT: storageRoot,
        GITY_WORKER__ENABLED: "false",
        GITY_SEARCH__INDEX_ENABLED: "false",
        GITY_SEARCH__INDEX_ROOT: searchRoot,
        GITY_E2E_BIN_ROOT: binRoot,
      },
    },
    {
      command: `pnpm exec vite --config vite.config.ts --host 127.0.0.1 --port ${webPort}`,
      cwd: __dirname,
      url: webBaseURL,
      reuseExistingServer: false,
      timeout: 240_000,
      env: {
        ...process.env,
        VITE_DEV_API_PROXY_TARGET: apiOrigin,
      },
    },
  ],
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
