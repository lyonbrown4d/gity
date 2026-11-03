import { mkdirSync, copyFileSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(scriptDir, "../../..");
const binRoot = process.env.GITY_E2E_BIN_ROOT ?? path.join(rootDir, ".tmp-e2e", "bin");
const serverBin = path.join(binRoot, process.platform === "win32" ? "gity-server-e2e.exe" : "gity-server-e2e");
const runnerBin = path.join(binRoot, process.platform === "win32" ? "gity-runner-e2e.exe" : "gity-runner-e2e");

mkdirSync(binRoot, { recursive: true });

const run = (command, args, options = {}) =>
  new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: rootDir,
      env: process.env,
      stdio: "inherit",
      shell: false,
      ...options,
    });

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${command} ${args.join(" ")} exited with code ${code ?? "null"} signal ${signal ?? "null"}`));
    });
  });

await run("cargo", ["build", "--locked", "-p", "gity-server", "-p", "gity-runner"]);
const targetRoot = process.env.CARGO_TARGET_DIR ?? path.join(rootDir, "target");
copyFileSync(path.join(targetRoot, "debug", process.platform === "win32" ? "gity-server.exe" : "gity-server"), serverBin);
copyFileSync(path.join(targetRoot, "debug", process.platform === "win32" ? "gity-runner.exe" : "gity-runner"), runnerBin);

let stopping = false;
const server = spawn(serverBin, ["standalone"], {
  cwd: rootDir,
  env: process.env,
  stdio: "inherit",
  shell: false,
});

const stop = () => {
  stopping = true;
  if (!server.pid || server.killed) {
    return;
  }
  if (process.platform === "win32") {
    spawnSync("taskkill", ["/pid", String(server.pid), "/T", "/F"], { stdio: "ignore" });
    return;
  }
  server.kill("SIGTERM");
};

process.on("SIGINT", stop);
process.on("SIGTERM", stop);
process.on("SIGHUP", stop);
process.on("exit", stop);

await new Promise((resolve, reject) => {
  server.on("error", reject);
  server.on("exit", (code, signal) => {
    if (stopping || signal) {
      resolve();
      return;
    }
    reject(new Error(`${serverBin} exited unexpectedly with code ${code ?? "null"}`));
  });
});
