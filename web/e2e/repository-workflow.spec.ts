import { expect, test } from "@playwright/test";
import { authenticatePage, seedProject } from "./support/api";

test("renders seeded project, repository content, and collaboration tabs", async ({ page, request }) => {
  const { session, organization, project, issue } = await seedProject(request);
  await authenticatePage(page, session);

  await page.goto("/app/projects");
  await expect(page.getByText(organization.name).first()).toBeVisible();
  await expect(page.getByRole("link", { name: project.name })).toBeVisible();

  await page.getByRole("link", { name: project.name }).click();
  await expect(page).toHaveURL(new RegExp(`/app/projects/${organization.id}/${project.id}`));
  await expect(page.getByRole("heading", { name: project.name })).toBeVisible();
  await expect(page.getByText(project.full_path, { exact: true })).toBeVisible();
  await expect(page.getByText("README.md")).toBeVisible();

  await page.getByRole("tab", { name: "Issues" }).click();
  await expect(page).toHaveURL(/tab=issues/);
  await expect(page.getByText(issue.title)).toBeVisible();

  await page.goto(`/app/projects/${organization.id}/${project.id}/issues/${issue.number}`);
  await expect(page.getByText(issue.title).first()).toBeVisible();
  await expect(page.getByText("Issue seeded for UI integration testing")).toBeVisible();

  await page.goto(`/app/projects/${organization.id}/${project.id}?tab=pipelines`);
  await expect(page.getByText(/Pipelines/).first()).toBeVisible();

  await page.getByRole("tab", { name: "Packages" }).click();
  await expect(page).toHaveURL(/tab=packages/);
  await expect(page.getByText(/Package Registry/).first()).toBeVisible();
});

