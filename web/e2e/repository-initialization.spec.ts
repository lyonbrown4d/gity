import { expect, test } from "@playwright/test";

test("creates a project with a catalog license and README initialization", async ({ page }, testInfo) => {
  const organization = { id: "51", key: "example-org", name: "Example Organization", role: "owner" };
  const catalog = [
    { id: "mit", name: "MIT License", spdx_id: "MIT" },
    { id: "apache-2.0", name: "Apache License 2.0", spdx_id: "Apache-2.0" },
  ];
  let submitted: Record<string, unknown> | undefined;
  await page.addInitScript(() => {
    localStorage.setItem("gity.access_token", "mock-access-token");
    localStorage.setItem("gity.refresh_token", "mock-refresh-token");
  });
  await page.route("**/api/v1/**", async (route) => {
    const url = new URL(route.request().url());
    const method = route.request().method();
    let data: unknown = [];
    if (url.pathname === "/api/v1/users/me") {
      data = { id: "41", username: "mock-owner", email: "owner@example.test", is_super_admin: false };
    } else if (url.pathname === "/api/v1/orgs") {
      data = [organization];
    } else if (url.pathname === "/api/v1/projects" && method === "POST") {
      submitted = route.request().postDataJSON() as Record<string, unknown>;
      data = {
        id: "90001",
        uuid: "00000000-0000-7000-8000-000000000001",
        organization_id: organization.id,
        key: "init-flow",
        path_key: "init-flow",
        full_path: `${organization.key}/init-flow`,
        name: "Initialization flow",
        description: null,
        visibility: "private",
        default_branch: "main",
        status: "active",
        clone_http_url: "http://localhost/example.git",
        created_at: "2026-11-03T00:00:00Z",
        updated_at: "2026-11-03T00:00:00Z",
      };
    } else if (url.pathname === "/api/v1/repository-templates/licenses") {
      data = catalog;
    } else if (url.pathname === "/api/v1/repository-templates/gitignores") {
      data = [{ id: "node", name: "Node" }, { id: "rust", name: "Rust" }];
    }
    await route.fulfill({
      json: {
        code: 0,
        message: "ok",
        data,
      },
    });
  });

  await page.goto("/app/projects");
  await page.getByRole("button", { name: "New Project" }).first().click();
  const dialog = page.getByRole("dialog", { name: "Create Project" });
  await expect(dialog.getByLabel("Add license")).toBeVisible();
  await dialog.getByLabel("Project key").fill("init-flow");
  await dialog.getByLabel("Project name").fill("Initialization flow");
  await dialog.getByLabel("Add .gitignore").click();
  await page.getByRole("option", { name: "Node", exact: true }).click();
  await dialog.getByLabel("Add license").click();
  await page.getByRole("option", { name: "MIT License" }).click();
  await dialog.getByLabel("Copyright holder").fill("Example Organization");
  await dialog.getByLabel("Initialize with a README").check();
  await page.screenshot({ path: testInfo.outputPath("repository-initialization-dialog.png") });
  await dialog.getByRole("button", { name: "Create" }).click();

  await expect(dialog).toBeHidden();
  expect(submitted).toMatchObject({
    organization_id: organization.id,
    key: "init-flow",
    path_key: "init-flow",
    gitignore_template: "node",
    license_template: "mit",
    initialize_with_readme: true,
    license_copyright_holder: "Example Organization",
  });
});
