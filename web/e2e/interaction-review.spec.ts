import { expect, test } from "@playwright/test";
import { readFileSync, mkdirSync } from "node:fs";
import path from "node:path";

test("review sections retain comment drafts and expose the real merge blocker", async ({ page, request }) => {
  const passwordFile = process.env.UI_TEST_PASSWORD_FILE;
  const org = process.env.UI_TEST_ORGANIZATION_ID;
  const project = process.env.UI_TEST_PROJECT_ID;
  if (!passwordFile || !org || !project) throw new Error("Provide isolated UI test environment settings");
  const output =
    process.env.UI_TEST_OUTPUT ?? path.resolve(import.meta.dirname, "../../target/ui-design-results/screenshots");
  mkdirSync(output, { recursive: true });
  const login = await request.post("/api/v1/auth/login", {
    data: {
      username: process.env.UI_TEST_USERNAME ?? "root",
      password: readFileSync(passwordFile, "utf8").trim(),
    },
  });
  expect(login.ok()).toBe(true);
  const { data: session } = (await login.json()) as { data: { token: string; refresh_token: string } };
  const headers = { Authorization: `Bearer ${session.token}` };
  const branch = `ui-review-${Date.now()}`;
  const api = `/api/v1/projects/${project}`;
  const createdBranch = await request.post(`${api}/repository/branches`, {
    headers,
    data: { name: branch, source_ref: "main" },
  });
  expect(createdBranch.ok()).toBe(true);
  const file = await request.post(`${api}/repository/files`, {
    headers,
    data: {
      branch_name: branch,
      path: "docs/review-proof.md",
      content: "# Review proof\n\nUse the current project and identity.\n",
      message: "Add review proof",
      author_name: "UI test",
      author_email: "ui@example.test",
      skip_pipeline: true,
    },
  });
  expect(file.ok()).toBe(true);
  const created = await request.post(`${api}/merge-requests`, {
    headers,
    data: {
      title: "Simplify project interactions",
      description: "Confirm review sections, draft retention and merge target.",
      source_branch: branch,
      target_branch: "main",
    },
  });
  expect(created.ok()).toBe(true);
  const { data: mr } = (await created.json()) as { data: { iid: number } };
  await page.goto("/login");
  await page.evaluate(({ token, refresh_token }) => {
    localStorage.setItem("gity.access_token", token);
    localStorage.setItem("gity.refresh_token", refresh_token);
  }, session);
  await page.goto(`/app/projects/${org}/${project}/merge-requests/${mr.iid}`);
  const changes = page.getByRole("tab", { name: "File changes", exact: true });
  await expect(changes).toHaveAttribute("aria-selected", "true");
  const panelId = await changes.getAttribute("aria-controls");
  await expect(page.locator(`[id="${panelId}"]`)).toHaveAttribute("role", "tabpanel");
  await expect(page.getByText("Use the current project and identity.", { exact: false }).first()).toBeVisible();
  await page.screenshot({
    path: path.join(output, "merge-request-review.png"),
    fullPage: true,
    animations: "disabled",
  });
  const discussion = page.getByRole("tab", { name: /^Discussion/ });
  await discussion.click();
  await expect(discussion).toHaveAttribute("aria-selected", "true");
  const draft = page.getByPlaceholder("Add a merge request comment");
  await draft.fill("Please keep this draft while checking the diff.");
  await changes.click();
  await expect(changes).toHaveAttribute("aria-selected", "true");
  await discussion.click();
  await expect(discussion).toHaveAttribute("aria-selected", "true");
  await expect(draft).toHaveValue("Please keep this draft while checking the diff.");
  await expect(page).toHaveURL(/review=discussion/);
  await page.reload();
  await expect(discussion).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("button", { name: "Merge", exact: true })).toBeDisabled();
  await expect(page.locator("#merge-disabled-reason")).toContainText("pipeline");
  await page.goto(`/app/projects/${org}/${project}/wiki`);
  await page.getByRole("button", { name: "New wiki page", exact: true }).click();
  const wiki = page.getByRole("dialog", { name: "New wiki page", exact: true });
  await expect(wiki.getByLabel("Author user ID")).toHaveCount(0);
  await wiki.getByLabel("Wiki page title", { exact: true }).fill(`Interaction guide ${Date.now()}`);
  await wiki
    .getByLabel("Content", { exact: true })
    .fill("# Interaction guide\n\nThe page author comes from the current session.");
  await wiki.getByRole("button", { name: "Create wiki page", exact: true }).click();
  await expect(wiki).toHaveCount(0);
  await expect(page.getByLabel("Editor user ID")).toHaveCount(0);
  await page.screenshot({ path: path.join(output, "wiki.png"), fullPage: true, animations: "disabled" });
  await page.goto("/admin/orgs");
  await page.getByRole("button", { name: "Add Member", exact: true }).click();
  const member = page.getByRole("dialog", { name: "Add Organization Member", exact: true });
  await expect(member.getByLabel("User ID", { exact: true })).toHaveCount(0);
  await member.getByRole("combobox", { name: "User", exact: true }).click();
  await page.getByRole("combobox", { name: "Search members", exact: true }).fill("root");
  await expect(page.getByRole("option").first()).toBeVisible();
  await page.getByRole("option").first().click();
  await expect(member.getByRole("button", { name: "Add Member", exact: true })).toBeEnabled();
  await page.screenshot({ path: path.join(output, "organization-member.png"), fullPage: true, animations: "disabled" });
  await member.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(member).toHaveCount(0);

});
