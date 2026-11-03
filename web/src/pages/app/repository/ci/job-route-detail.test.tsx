// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { useCustom } from "@refinedev/core";
import { RepositoryJobsTab } from "../repository-jobs-tab";
import { buildRepositoryPermissions } from "../repository-permissions";
vi.mock("@refinedev/core", () => ({ useCustom: vi.fn(), useCustomMutation: vi.fn() }));
vi.mock("./job-detail", () => ({ JobDetail: ({ job }: { job: { id: string } }) => <p>Job details {job.id}</p> }));
afterEach(cleanup);
beforeEach(() => {
  vi.mocked(useCustom).mockReset();
});
const setup = (failed = false) => {
  vi.mocked(useCustom).mockImplementation(
    (options) =>
      ({
        result: {
          data:
            options.url.endsWith("/jobs/9223372036854775806") && !failed
              ? { id: "9223372036854775806", status: "failed" }
              : undefined,
        },
        query: {
          isLoading: false,
          error: failed && options.url.endsWith("/jobs/9223372036854775806") ? new Error("Job not found") : null,
        },
      }) as unknown as ReturnType<typeof useCustom>,
  );
  const router = createMemoryRouter(
    [
      {
        path: "/app/projects/:organizationId/:projectId/jobs/:jobId",
        element: (
          <RepositoryJobsTab
            repoId="31"
            permissions={buildRepositoryPermissions("developer", false)}
            t={(key) => key}
            onError={() => {}}
          />
        ),
      },
    ],
    { initialEntries: ["/app/projects/21/31/jobs/9223372036854775806"] },
  );
  render(<RouterProvider router={router} />);
};
describe("Canonical job detail route", () => {
  it("loads the exact deep-linked job even when it is absent from the latest jobs list", () => {
    setup();
    expect(screen.getByText("Job details 9223372036854775806")).toBeTruthy();
    const requests = vi.mocked(useCustom).mock.calls.map(([options]) => options);
    expect(requests.find((request) => request.url === "/projects/31/jobs")?.queryOptions?.enabled).toBe(false);
    expect(requests.some((request) => request.url === "/projects/31/jobs/9223372036854775806")).toBe(true);
  });
  it("reports a missing or inaccessible deep-linked job instead of showing an empty jobs list", () => {
    setup(true);
    expect(screen.getByRole("alert").textContent).toBe("Job not found");
    expect(screen.queryByText("No jobs yet.")).toBeNull();
    expect(screen.queryByText("Job details 9223372036854775806")).toBeNull();
  });
});
