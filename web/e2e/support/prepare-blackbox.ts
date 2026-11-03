import { spawnSync } from "node:child_process";
import { existsSync, copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";

const prepareBlackbox = (): void => {
  const runnerBin = process.env.GITY_E2E_RUNNER_BIN;
  if (!runnerBin || existsSync(runnerBin)) {
    return;
  }

  const repoRoot = path.resolve(import.meta.dirname, "../../..");
  const result = spawnSync("cargo", ["build", "--locked", "-p", "gity-runner"], {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`failed to build blackbox runner: cargo exited with status ${result.status ?? "unknown"}`);
  }
  mkdirSync(path.dirname(runnerBin), { recursive: true });
  copyFileSync(path.join(process.env.CARGO_TARGET_DIR ?? path.join(repoRoot, "target"), "debug", process.platform === "win32" ? "gity-runner.exe" : "gity-runner"), runnerBin);
};
export default prepareBlackbox;
