import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";

export default function prepareBlackbox(): void {
  const runnerBin = process.env.GITY_E2E_RUNNER_BIN;
  if (!runnerBin || existsSync(runnerBin)) {
    return;
  }

  const repoRoot = path.resolve(import.meta.dirname, "../../..");
  const goBin = process.platform === "win32" ? "go.exe" : "go";
  const result = spawnSync(goBin, ["build", "-o", runnerBin, "./cmd/runner"], {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`failed to build blackbox runner: go exited with status ${result.status ?? "unknown"}`);
  }
}
