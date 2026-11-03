import { expect, test, type Route } from "@playwright/test";

test("global quick jump returns issue and pinned code results and clears permission failures", async ({ page }, testInfo) => {
  const project = {
    id: "22",
    organization_id: "11",
    uuid: "project-uuid",
    key: "platform",
    full_path: "team/platform",
    name: "Platform",
    description: "Main project",
    visibility: "private",
    default_branch: "main",
    status: "active",
    clone_http_url: "https://example.test/team/platform.git",
  };
  const searches: Array<{ query: string; types: string[] }> = [];
  const blobQueries: URLSearchParams[] = [];
  const commit = "abcdef0123456789abcdef0123456789abcdef01";
  await page.addInitScript(() => {
    localStorage.setItem("gity.access_token", "access-token");
    localStorage.setItem("gity.refresh_token", "refresh-token");
  });
  await page.route("**/api/v1/**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname.replace("/api/v1", "");
    if (path === "/users/me") {
      await fulfill(route, { id: "7", username: "search-user", email: "search@example.test", status: "active", is_super_admin: false });
      return;
    }
    if (path === "/orgs") {
      await fulfill(route, [{ id: "11", key: "team", name: "Team", role: "maintainer" }]);
      return;
    }
    if (path === "/orgs/11") {
      await fulfill(route, { id: "11", key: "team", name: "Team", role: "maintainer" });
      return;
    }
    if (path === "/projects" || path === "/projects/22") {
      await fulfill(route, [project]);
      return;
    }
    if (path === "/projects/22/permissions") {
      await fulfill(route, { role_level: 40, repository_push: true, issue_create: true });
      return;
    }
    if (path === "/projects/22/repository/blob") {
      blobQueries.push(url.searchParams);
      await fulfill(route, {
        name: "kernel.rs",
        path: "src/kernel.rs",
        content: "fn start_kernel() {\n  boot_check();\n}\n",
        encoding: "utf-8",
        is_binary: false,
        size: 40,
      });
      return;
    }
    if (path === "/todos" || path === "/activity") {
      await fulfill(route, []);
      return;
    }
    if (path === "/search" && request.method() === "POST") {
      const payload = request.postDataJSON() as { query: string; types: string[] };
      searches.push(payload);
      if (payload.query === "restricted") {
        await route.fulfill({ status: 403, json: { code: 403, message: "Forbidden", data: null } });
        return;
      }
      await fulfill(route, {
        groups: [
          { kind: "issue", items: [{ kind: "issue", organization_id: "11", project_id: "22", project_path: "team/platform", resource_id: "301", title: "Kernel panic on startup", snippet: "Fails after the boot check", path: null, commit_sha: null, line: null, issue_iid: "8", state: "open" }] },
          { kind: "code", items: [{ kind: "code", organization_id: "11", project_id: "22", project_path: "team/platform", resource_id: "blob-sha", title: "src/kernel.rs", snippet: "fn start_kernel()", path: "src/kernel.rs", commit_sha: commit, line: 2, issue_iid: null, state: null }] },
        ],
        next_cursor: null,
        partial: false,
        indexing_eventually_consistent: true,
        max_code_files_per_project: 1000,
        max_code_file_size_bytes: 100000,
      });
      return;
    }
    await fulfill(route, []);
  });

  await page.goto("/app/dashboard");
  await page.getByRole("button", { name: "Quick jump" }).click();
  const input = page.getByPlaceholder("Search projects, issues, merge requests, files...");
  await input.fill("kernel");
  const issueResult = page.getByRole("option", { name: /Kernel panic on startup/ });
  const codeResult = page.getByRole("option", { name: /src\/kernel\.rs/ });
  await expect(issueResult).toBeVisible();
  await expect(codeResult).toBeVisible();
  expect(searches.some(({ query, types }) => query === "kernel" && types.includes("code") && types.includes("issue"))).toBe(true);
  await page.screenshot({ path: testInfo.outputPath("global-quick-jump-results.png") });

  await codeResult.click();
  await expect(page).toHaveURL(new RegExp(`/app/projects/11/22\\?tab=code&commit_sha=${commit}&path=src%2Fkernel\\.rs&line=2#L2$`));
  await expect(page.getByText(`${commit.slice(0, 12)} · src/kernel.rs:2`)).toBeVisible();
  await expect.poll(() => blobQueries.length).toBeGreaterThan(0);
  expect(blobQueries[0].get("ref_name")).toBe(commit);
  expect(blobQueries[0].get("path")).toBe("src/kernel.rs");
  await expect(page.getByText("boot_check();")).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath("pinned-source.png") });

  await page.getByRole("button", { name: "Quick jump" }).click();
  await page.getByPlaceholder("Search projects, issues, merge requests, files...").fill("restricted");
  await expect(page.getByRole("alert")).toContainText("Global search is unavailable");
  await expect(page.getByRole("option", { name: /Kernel panic on startup/ })).toHaveCount(0);
});

const fulfill = async (route: Route, data: unknown): Promise<void> => {
  await route.fulfill({ status: 200, json: { code: 0, message: "ok", data } });
};
