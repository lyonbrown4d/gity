import { expect, test } from "@playwright/test";
import { authenticatePage, createOrganization, createProject, loginByAPI, uniqueKey } from "./support/api";

test("switches the secondary navigation with each primary workspace", async ({ page, request }) => {
  const session = await loginByAPI(request, uniqueKey("navigation-user"));
  const organization = await createOrganization(request, session);
  const project = await createProject(request, session, organization);
  await authenticatePage(page, session);

  await page.goto("/app/dashboard");
  const sidebar = page.locator('[data-sidebar="sidebar"]');
  await expect(sidebar.getByRole("link", { name: "Dashboard", exact: true })).toBeVisible();
  await expect(sidebar.getByRole("link", { name: "Profile", exact: true })).toBeVisible();

  await page.getByRole("link", { name: "Projects", exact: true }).click();
  await expect(page).toHaveURL(/\/app\/projects$/);
  await expect(page.getByRole("link", { name: "All projects", exact: true })).toBeVisible();
  await expect(sidebar.getByRole("link", { name: project.name, exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Profile", exact: true })).toHaveCount(0);

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page).toHaveURL(/\/app\/todos$/);
  await expect(page.getByRole("link", { name: "Pending", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Done", exact: true })).toBeVisible();

  await page.getByRole("link", { name: "Done", exact: true }).click();
  await expect(page).toHaveURL(/\/app\/todos\?state=done$/);
  await expect(page.getByRole("tab", { name: "Done", exact: true })).toHaveAttribute("data-state", "active");
});

test("opens the command palette from the keyboard and switches projects", async ({ page, request }) => {
  const session = await loginByAPI(request, uniqueKey("command-user"));
  const organization = await createOrganization(request, session);
  const project = await createProject(request, session, organization);
  await authenticatePage(page, session);
  await page.goto("/app/dashboard");

  await expect(page.getByRole("button", { name: /Quick jump/ })).toBeVisible();
  await page.keyboard.press("Control+K");
  const palette = page.getByRole("dialog", { name: "Quick jump" });
  await expect(palette).toBeVisible();
  await palette.getByPlaceholder("Search projects, issues, merge requests, files...").fill(project.name);
  await palette.getByRole("option", { name: new RegExp(project.name) }).click();

  await expect(page).toHaveURL(new RegExp(`/app/projects/${organization.id}/${project.id}$`));
  await expect(palette).toBeHidden();

  await page.keyboard.press("Control+K");
  await palette.getByPlaceholder("Search projects, issues, merge requests, files...").fill("merge requests");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\?tab=merge-requests$/);
});
