import { expect, type Locator, type Page, test } from "@playwright/test";
import { authenticatePage, createBranch, createFileCommit, createIssueComment, seedProject, uniqueKey } from "./support/api";

test("drives repository collaboration across search, issues, wiki, pipelines, and merge requests", async ({ page, request }) => {
  const { session, organization, project } = await seedProject(request);
  const searchToken = uniqueKey("search-token");
  const featureBranch = `feature/${uniqueKey("review")}`;
  const issueTitle = `Browser issue ${uniqueKey("issue")}`;
  const issueComment = `Issue comment ${uniqueKey("comment")}`;
  const issueLabel = `triage-${uniqueKey("label")}`;
  const wikiTitle = `Runbook ${uniqueKey("wiki")}`;
  const wikiSlug = wikiTitle.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
  const mrTitle = `Review ${uniqueKey("mr")}`;
  const mrComment = `MR discussion ${uniqueKey("mr-comment")}`;
  const projectURL = `/app/projects/${organization.id}/${project.id}`;

  await createFileCommit(request, session, project, {
    path: "docs/search.md",
    content: `# Search fixture\n\nThe unique browser-search marker is ${searchToken}.\n`,
    message: "Add searchable fixture",
  });
  await createBranch(request, session, project, featureBranch);
  await createFileCommit(request, session, project, {
    branchName: featureBranch,
    path: "src/review-flow.go",
    content: "package main\n\nfunc reviewFlow() string { return \"ready\" }\n",
    message: "Add review flow fixture",
  });

  await authenticatePage(page, session);

  await page.goto(projectURL);
  await expect(page.getByRole("heading", { name: project.name })).toBeVisible();
  await expect(page.getByText(project.full_path, { exact: true })).toBeVisible();

  await typeInto(page.getByPlaceholder("Search text"), searchToken);
  await page.getByRole("button", { name: "Search" }).click();
  await expect(page.getByText("docs/search.md").first()).toBeVisible();

  await page.getByRole("tab", { name: "Issues" }).click();
  await page.getByRole("button", { name: "New issue" }).click();
  await typeInto(page.getByPlaceholder("Issue title"), issueTitle);
  await page.getByRole("button", { name: "Create issue" }).click();

  await expect(page).toHaveURL(/\/issues\/\d+$/);
  await expect(page.getByText(issueTitle).first()).toBeVisible();
  await expect(page.getByRole("button", { name: "Comment" })).toBeDisabled();
  const issueNumber = issueNumberFromCurrentURL(page);
  await createIssueComment(request, session, project, issueNumber, issueComment);
  await page.reload();
  await expect(page.getByText("1 comment(s)")).toBeVisible();
  await expect(page.getByText(issueComment).first()).toBeVisible();

  await typeInto(page.getByLabel("Label name"), issueLabel);
  await page.getByLabel("Label color").fill("#f59e0b");
  await page.locator("aside").getByRole("button", { name: "Add" }).last().click();
  await expect(page.getByText(issueLabel)).toBeVisible();

  await page.getByRole("button", { name: "Close issue" }).click();
  await expect(page.getByRole("button", { name: "Reopen issue" })).toBeVisible();
  await expect(page.getByText("Closed").first()).toBeVisible();

  await page.goto(`${projectURL}?tab=wiki`);
  await page.getByRole("button", { name: "New wiki page" }).click();
  await typeInto(page.getByLabel("Wiki page title"), wikiTitle);
  await typeInto(page.getByLabel("Slug optional"), wikiSlug);
  await page.getByLabel("Author user ID").fill(session.user_id);
  await page.getByLabel("Content").fill(`# ${wikiTitle}\n\nCreated from the browser integration flow.`);
  await page.getByRole("button", { name: "Create wiki page" }).click();
  await expect(page.getByText(wikiTitle).first()).toBeVisible();

  const wikiEditor = page.getByLabel("Wiki page content");
  await expect(wikiEditor).toHaveValue(`# ${wikiTitle}\n\nCreated from the browser integration flow.`);
  await wikiEditor.fill(`# ${wikiTitle}\n\nUpdated browser proof.`);
  await expect(wikiEditor).toHaveValue(`# ${wikiTitle}\n\nUpdated browser proof.`);
  await page.getByRole("button", { name: "Save wiki page" }).click();
  await expect(page.locator("article.markdown-body").getByText("Updated browser proof.", { exact: true })).toBeVisible();

  await page.goto(`${projectURL}?tab=pipelines`);
  await page.getByRole("button", { name: "New pipeline" }).click();
  await page.getByLabel("Ref").fill(project.default_branch || "main");
  await page.getByLabel("Plano CI config").fill(`pipeline {\n  name = "browser-flow"\n}\n\nstage test {\n  run {\n    shell("echo browser-flow")\n  }\n}\n`);
  await page.getByRole("button", { name: "Create pipeline" }).click();
  await expect(page.getByRole("button", { name: /#\d+ browser-flow/ })).toBeVisible();
  await expect(page.getByText("Pipeline jobs")).toBeVisible();

  await page.goto(`${projectURL}?tab=merge-requests`);
  await page.getByRole("button", { name: "New merge request" }).click();
  await typeInto(page.getByPlaceholder("Merge request title"), mrTitle);
  await page.getByPlaceholder("Describe the merge request (optional)").fill("Created by the browser collaboration flow.");
  await selectRadixOption(page, "Source branch", featureBranch);
  await page.getByRole("button", { name: "Create merge request" }).click();

  await expect(page.getByText(mrTitle).first()).toBeVisible();
  await expect(page.getByText(`${featureBranch} → ${project.default_branch || "main"}`).first()).toBeVisible();

  await page.getByPlaceholder("Add a merge request comment").fill(mrComment);
  await page.getByRole("button", { name: "Comment" }).last().click();
  await expect(page.getByText(mrComment)).toBeVisible();

  await page.getByRole("button", { name: "Approve" }).click();
  await expect(page.getByRole("button", { name: "Unapprove" })).toBeVisible();
  await expect(page.getByText("You approved").first()).toBeVisible();
});

function issueNumberFromCurrentURL(page: Page): number {
  const match = page.url().match(/\/issues\/(\d+)$/);
  if (!match) {
    throw new Error(`Issue detail URL did not contain an issue number: ${page.url()}`);
  }
  return Number.parseInt(match[1], 10);
}

async function selectRadixOption(page: Page, label: string, option: string): Promise<void> {
  await page.getByLabel(label).click();
  await page.getByRole("option", { name: option, exact: true }).click();
}

async function typeInto(locator: Locator, value: string): Promise<void> {
  await locator.click();
  await locator.pressSequentially(value);
  await expect(locator).toHaveValue(value);
}


