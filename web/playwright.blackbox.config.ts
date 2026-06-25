import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { defineConfig, devices } from "@playwright/test";

const webBaseURL = (process.env.E2E_WEB_BASE_URL ?? process.env.BLACKBOX_WEB_BASE_URL ?? "http://127.0.0.1:18080").replace(
  /\/+$/,
  "",
);
const apiBaseURL = (process.env.E2E_API_BASE_URL ?? `${webBaseURL}/api/v1`).replace(/\/+$/, "");
const runRoot = process.env.GITY_E2E_ROOT
  ? path.resolve(process.env.GITY_E2E_ROOT)
  : fs.mkdtempSync(path.join(os.tmpdir(), "gity-blackbox-e2e-"));
const binRoot = path.join(runRoot, "bin");
const runnerBin =
  process.env.GITY_E2E_RUNNER_BIN ??
  path.join(binRoot, process.platform === "win32" ? "gity-runner-e2e.exe" : "gity-runner-e2e");

for (const dir of [runRoot, binRoot]) {
  fs.mkdirSync(dir, { recursive: true });
}

process.env.E2E_WEB_BASE_URL = webBaseURL;
process.env.E2E_API_BASE_URL = apiBaseURL;
process.env.GITY_E2E_ROOT = runRoot;
process.env.GITY_E2E_BIN_ROOT = binRoot;
process.env.GITY_E2E_REPO_ROOT = process.env.GITY_E2E_REPO_ROOT ?? "";
process.env.GITY_E2E_RUNNER_BIN = runnerBin;

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
    ["html", { outputFolder: "../.tmp-e2e/blackbox-playwright-report", open: "never" }],
  ],
  outputDir: "../.tmp-e2e/blackbox-playwright-results",
  use: {
    baseURL: webBaseURL,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
