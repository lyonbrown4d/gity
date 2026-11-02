import { mkdirSync } from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { expect, type Locator, test } from "@playwright/test";
import { authenticatePage, seedProject, uniqueKey } from "./support/api";

const apiBaseURL = process.env.E2E_API_BASE_URL ?? "http://127.0.0.1:19083/api/v1";
const runnerBin = process.env.GITY_E2E_RUNNER_BIN;
const runnerWorkRoot = path.join(process.env.GITY_E2E_ROOT ?? path.join(process.cwd(), "..", ".tmp-e2e"), "runner-work");
const repoRoot = process.env.GITY_E2E_REPO_ROOT ?? "";
const testFileDir = path.dirname(fileURLToPath(import.meta.url));

test("registers a runner and executes a pipeline job through the runner agent", async ({ page, request }) => {
	const { session, organization, project } = await seedProject(request);
	const projectURL = `/app/projects/${organization.id}/${project.id}`;
	const runnerName = uniqueKey("host-runner");
	const outputMarker = uniqueKey("runner-output");

	await authenticatePage(page, session);

	await page.goto(`${projectURL}?tab=runners`);
	await page.getByRole("button", { name: "Register runner" }).click();
	await typeInto(page.getByLabel("Runner name"), runnerName);
	await page.getByLabel("Runner tags").fill("linux,go");
	await page.getByLabel("Description").fill("Playwright host runner");
	await page.getByRole("button", { name: "Register runner" }).last().click();
	await expect(page.getByText("Runner token generated")).toBeVisible();
	await expect(page.getByText(runnerName)).toBeVisible();
	const token = await runnerToken(page.locator("code"));

	await page.goto(`${projectURL}?tab=pipelines`);
	await page.getByRole("button", { name: "New pipeline" }).click();
	await page.getByLabel("Ref").fill(project.default_branch || "main");
	await page.getByLabel("Plano CI config").fill(`pipeline {
  name = "runner-e2e"
}

stage test {
  tags = ["linux", "go"]
  run {
    shell("echo ${outputMarker}")
  }
}
`);
	await page.getByRole("button", { name: "Create pipeline" }).click();
	await expect(page.getByRole("button", { name: /#\d+ runner-e2e/ })).toBeVisible();

	await runRunnerOnce(token, outputMarker);

	await page.goto(`${projectURL}?tab=pipelines`);
	await page.getByRole("button", { name: /#\d+ runner-e2e/ }).click();
	await page.getByRole("button", { name: "Reload pipeline" }).click();
	await expect(page.getByText("Completed successfully.")).toBeVisible();
	await page.getByRole("button", { name: "Logs" }).click();
	await expect(page.getByText(outputMarker, { exact: true })).toBeVisible();
});

async function runnerToken(codes: Locator): Promise<string> {
	const count = await codes.count();
	for (let index = 0; index < count; index += 1) {
		const text = (await codes.nth(index).textContent())?.trim() ?? "";
		if (text.length > 20) {
			return text;
		}
	}
	throw new Error("Runner token was not rendered in the registration result.");
}

async function runRunnerOnce(token: string, marker: string): Promise<void> {
	if (!runnerBin) {
		throw new Error("GITY_E2E_RUNNER_BIN is not set.");
	}
	const workDir = path.join(runnerWorkRoot, marker);
	mkdirSync(workDir, { recursive: true });
	await spawnRunner([
		"--server",
		apiBaseURL,
		"--token",
		token,
		"--workdir",
		workDir,
		"--repo-root",
		repoRoot,
		"--execution-mode",
		"host",
		"--allowed-shells",
		"sh,bash,powershell,pwsh,cmd",
		"--once",
	]);
}

function spawnRunner(args: string[]): Promise<void> {
	return new Promise((resolve, reject) => {
		const child = spawn(runnerBin!, args, {
			cwd: path.resolve(testFileDir, "../.."),
			env: process.env,
			stdio: ["ignore", "pipe", "pipe"],
			shell: false,
		});
		let output = "";
		child.stdout.on("data", (chunk: Buffer) => {
			output += chunk.toString();
		});
		child.stderr.on("data", (chunk: Buffer) => {
			output += chunk.toString();
		});
		child.on("error", reject);
		child.on("exit", (code, signal) => {
			if (code === 0) {
				resolve();
				return;
			}
			reject(new Error(`runner exited with code ${code ?? "null"} signal ${signal ?? "null"}\n${output}`));
		});
	});
}

async function typeInto(locator: Locator, value: string): Promise<void> {
	await locator.click();
	await locator.pressSequentially(value);
	await expect(locator).toHaveValue(value);
}
