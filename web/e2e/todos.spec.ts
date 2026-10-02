import { expect, test, type Route } from "@playwright/test";

test("shows generated todos and marks them done from the inbox", async ({ page }) => {
  const project = { id: "22", organizationId: "11", fullPath: "acme/platform" };
  const todoTitle = `Project created: ${project.fullPath}`;
  let isDone = false;
  const todoRequests: string[] = [];

  await page.addInitScript(() => {
    localStorage.setItem("gity.access_token", "access-token");
    localStorage.setItem("gity.refresh_token", "refresh-token");
  });
  await page.route("**/api/v1/**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname.replace("/api/v1", "");

    if (path === "/users/me") {
      await fulfill(route, {
        id: "7",
        username: "todo-user",
        email: "todo-user@example.com",
        status: "active",
        is_super_admin: false,
      });
      return;
    }
    if (path === "/orgs" || path === "/projects") {
      await fulfill(route, []);
      return;
    }
    if (path === "/todos/42/done" && request.method() === "POST") {
      todoRequests.push(`${request.method()} ${path}`);
      isDone = true;
      await fulfill(route, { id: 42, state: "done" });
      return;
    }
    if (path === "/todos" && request.method() === "GET") {
      const state = url.searchParams.get("state");
      todoRequests.push(`${request.method()} ${path}?${url.searchParams.toString()}`);
      const matchesState = state === (isDone ? "done" : "pending");
      await fulfill(route, matchesState ? [todoFixture(isDone, todoTitle, project)] : []);
      return;
    }
    await route.fulfill({ status: 404, json: { code: 404, message: `Unhandled ${path}`, data: null } });
  });

  await page.goto("/app/dashboard");

  await expect(page.getByRole("link", { name: "Open Inbox" })).toBeVisible();
  const pendingMetric = page.locator(".gity-metric-card").filter({ hasText: "Pending Todos" });
  await expect(pendingMetric).toContainText("1");

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page).toHaveURL(/\/app\/todos$/);
  await expect(page.getByRole("heading", { name: "Inbox" })).toBeVisible();
  await expect(page.getByText(todoTitle)).toBeVisible();
  await expect(page.getByRole("link", { name: "Open", exact: true })).toHaveAttribute(
    "href",
    `/app/projects/${project.organizationId}/${project.id}`,
  );

  await page.getByRole("button", { name: "Mark done" }).click();
  await expect(page.getByText(todoTitle)).toHaveCount(0);
  await expect(page.getByText("You're all caught up.")).toBeVisible();

  await page.getByRole("tab", { name: "Done" }).click();
  await expect(page.getByText(todoTitle)).toBeVisible();

  await page.getByRole("button", { name: "Quick jump" }).click();
  await page.getByPlaceholder("Search projects, issues, merge requests, files...").fill("inbox");
  await expect(page.getByRole("button", { name: /Inbox/ })).toBeVisible();

  expect(todoRequests).toContain("GET /todos?state=pending&limit=100");
  expect(todoRequests).toContain("POST /todos/42/done");
  expect(todoRequests).toContain("GET /todos?state=done&limit=100");
});

function todoFixture(
  isDone: boolean,
  title: string,
  project: { id: string; organizationId: string },
) {
  return {
    id: 42,
    user_id: 7,
    organization_id: Number(project.organizationId),
    project_id: Number(project.id),
    kind: "project_created",
    state: isDone ? "done" : "pending",
    target_type: "project",
    target_id: project.id,
    title,
    summary: "A new project is ready for collaboration.",
    action_url: `/app/projects/${project.organizationId}/${project.id}`,
    created_at: "2026-10-02T08:00:00Z",
    updated_at: "2026-10-02T08:00:00Z",
    ...(isDone ? { done_at: "2026-10-02T08:05:00Z" } : {}),
  };
}

async function fulfill(route: Route, data: unknown): Promise<void> {
  await route.fulfill({ status: 200, json: { code: 0, message: "ok", data } });
}
