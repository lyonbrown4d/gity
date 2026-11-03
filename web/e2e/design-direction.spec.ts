import { expect, test } from "@playwright/test";
import { readFileSync, readdirSync, mkdirSync } from "node:fs";
import path from "node:path";

const localeRoot = path.resolve(import.meta.dirname, "../../crates/gity-i18n/locales");
const entries = Object.assign({}, ...readdirSync(localeRoot).filter((file) => file.endsWith(".json"))
  .map((file) => JSON.parse(readFileSync(path.join(localeRoot, file), "utf8")))) as Record<string, { zh?: string }>;
const t = (key: string) => entries[key]?.zh ?? key;

test("workspace, configurable themes, repository source and routed CI editing", async ({ page }) => {
  const passwordFile = process.env.UI_TEST_PASSWORD_FILE;
  const org = process.env.UI_TEST_ORGANIZATION_ID;
  const project = process.env.UI_TEST_PROJECT_ID;
  if (!passwordFile || !org || !project) throw new Error("Provide UI_TEST_PASSWORD_FILE, UI_TEST_ORGANIZATION_ID and UI_TEST_PROJECT_ID for an isolated test environment");
  const output = process.env.UI_TEST_OUTPUT ?? path.resolve(import.meta.dirname, "../../target/ui-design-results/screenshots");
  mkdirSync(output, { recursive: true });
  const externalAssets: string[] = [];
  page.on("request", (request) => {
    if (["script", "font", "stylesheet"].includes(request.resourceType()) && /^https?:/.test(request.url())
      && new URL(request.url()).origin !== new URL(page.url()).origin) externalAssets.push(request.url());
  });
  await page.goto("/login");
  await page.getByRole("textbox", { name: "Username / Email", exact: true }).fill(process.env.UI_TEST_USERNAME ?? "root");
  await page.getByRole("textbox", { name: "Password", exact: true }).fill(readFileSync(passwordFile, "utf8").trim());
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(page).toHaveURL(/\/app\/dashboard$/);
  await page.getByRole("button", { name: "Palette settings", exact: true }).click();
  await page.getByRole("button", { name: "Light", exact: true }).click();
  await page.getByRole("button", { name: "Iris", exact: true }).click();
  await page.getByRole("button", { name: "Close", exact: true }).click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-palette", "iris");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await page.getByRole("button", { name: "Palette settings", exact: true }).click();
  await page.getByRole("button", { name: "Mineral", exact: true }).click();
  await page.getByRole("button", { name: "Close", exact: true }).click();
  await page.getByRole("button", { name: "Switch language", exact: true }).click();
  await expect(page.getByRole("heading", { name: t("Workspace"), exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: t("Recent projects"), exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: t("Recent activity"), exact: true })).toBeVisible();
  await page.screenshot({ path: path.join(output, "home.png"), fullPage: true });
  await page.getByRole("link", { name: t("New Project"), exact: true }).click();
  await expect(page.getByRole("dialog", { name: t("Create Project") })).toBeVisible();
  await page.getByRole("button", { name: t("Cancel"), exact: true }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const base = `/app/projects/${encodeURIComponent(org)}/${encodeURIComponent(project)}`;
  await page.goto(`${base}/code`);
  await expect(page.locator("#repository-title")).toHaveText("Gity Platform");
  await expect(page.getByText("README.md", { exact: true }).first()).toBeVisible();
  await page.screenshot({ path: path.join(output, "repository.png"), fullPage: true });
  await page.goto(`${base}/pipelines/editor`);
  await expect(page.locator(".react-flow__node-job")).toHaveCount(4);
  await page.locator(".react-flow__node-job").filter({ hasText: "build" }).click();
  await expect(page.getByRole("textbox", { name: t("Task name"), exact: true })).toHaveValue("build");
  await page.screenshot({ path: path.join(output, "ci-editor.png"), fullPage: true });
  await page.getByRole("button", { name: "YAML", exact: true }).first().click();
  await expect(page.locator(".monaco-editor").first()).toBeVisible();
  await page.getByRole("button", { name: t("Canvas"), exact: true }).click();
  await page.getByRole("textbox", { name: t("Task name"), exact: true }).fill("build-qa");
  await page.getByRole("button", { name: t("Apply task changes"), exact: true }).click();
  await page.getByRole("link", { name: t("Code"), exact: true }).click();
  await expect(page.getByRole("dialog", { name: t("Unsaved changes") })).toBeVisible();
  await page.getByRole("button", { name: t("Keep editing"), exact: true }).click();
  await expect(page).toHaveURL(new RegExp("/pipelines/editor$"));
  await page.getByRole("link", { name: t("Code"), exact: true }).click();
  await page.getByRole("button", { name: t("Discard changes"), exact: true }).click();
  await expect(page).toHaveURL(new RegExp("/code$"));
  expect(externalAssets, "Editor resources must load from the local bundle").toEqual([]);
});
