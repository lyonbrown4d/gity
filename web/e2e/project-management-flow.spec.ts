import { expect, type Locator, test } from "@playwright/test";
import { authenticatePage, createOrganization, loginByAPI, uniqueKey } from "./support/api";

test("creates, opens, and deletes a project through the workspace UI", async ({ page, request }) => {
  const session = await loginByAPI(request, uniqueKey("ui-project-owner"));
  const organization = await createOrganization(request, session);
  await authenticatePage(page, session);

  const projectKey = uniqueKey("ui-project");
  const projectName = `UI Project ${projectKey}`;
  const expectedFullPath = `${organization.key}/${projectKey}`;

  await page.goto("/app/projects");
  await expect(page.getByText(organization.name).first()).toBeVisible();

  await page.getByRole("button", { name: "New Project" }).first().click();
  const dialog = page.getByRole("dialog", { name: "Create Project" });
  await expect(dialog).toBeVisible();
  await typeInto(dialog.getByLabel("Project key"), projectKey);
  await typeInto(dialog.getByLabel("Project name"), projectName);
  await typeInto(dialog.getByLabel("Description"), "Created end-to-end from the browser workspace.");
  await dialog.getByLabel("Default branch").fill("main");
  await dialog.getByRole("button", { name: "Create" }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByRole("link", { name: projectName })).toBeVisible();
  await expect(page.getByText(expectedFullPath).first()).toBeVisible();

  await page.getByRole("link", { name: projectName }).click();
  await expect(page.getByText(projectName).first()).toBeVisible();
  await expect(page.getByText(expectedFullPath, { exact: true })).toBeVisible();

  await page.goto("/app/projects");
  const projectCard = page.locator(".group").filter({ has: page.getByRole("link", { name: projectName }) }).first();
  await projectCard.getByRole("button", { name: "Delete" }).click();

  const confirmDialog = page.getByRole("dialog", { name: new RegExp(`Delete project "${projectName}"`) });
  await expect(confirmDialog).toBeVisible();
  await confirmDialog.getByPlaceholder(expectedFullPath).fill(expectedFullPath);
  await confirmDialog.getByRole("button", { name: "Delete" }).click();

  await expect(confirmDialog).toBeHidden();
  await expect(page.getByRole("link", { name: projectName })).toHaveCount(0);
});

async function typeInto(locator: Locator, value: string): Promise<void> {
  await locator.click();
  await locator.pressSequentially(value);
  await expect(locator).toHaveValue(value);
}

