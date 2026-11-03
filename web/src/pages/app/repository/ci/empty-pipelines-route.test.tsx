// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { Refine } from "@refinedev/core";
import { QueryClient } from "@tanstack/react-query";
import { dataProvider } from "@/providers/data-provider";
import { customFetch } from "@/lib/orval-fetch";
import { RepositoryPipelinesTab } from "../repository-pipelines-tab";
import { buildRepositoryPermissions } from "../repository-permissions";
vi.mock("@/lib/api", () => ({ getApiBaseUrl: () => "/api/v1" }));
vi.mock("@/lib/orval-fetch", () => ({ customFetch: vi.fn() }));
vi.mock("@/lib/theme", () => ({ useTheme: () => ({ editorTheme: "gity-light" }) }));
vi.mock("./pipeline-graph", () => ({ PipelineGraph: () => <div>Pipeline canvas</div> }));
const transport = vi.mocked(customFetch);
afterEach(() => {
  cleanup();
  sessionStorage.clear();
});
beforeEach(() => {
  transport.mockReset();
  transport.mockImplementation(async (url) => {
    let data: unknown = [];
    if (url.endsWith("/repository/branches")) data = [{ name: "main", last_commit_sha: "head", is_protected: false }];
    if (url.includes("/repository/blob?"))
      data = {
        path: ".gity-ci.yml",
        content: "jobs:\n  test:\n    script: cargo test\n",
        is_binary: false,
        encoding: "utf-8",
        size: 40,
      };
    return { code: 0, message: "ok", data };
  });
});
const setup = (path: string) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
  const router = createMemoryRouter(
    [
      {
        path: "/app/projects/:organizationId/:projectId/pipelines/*",
        element: (
          <Refine
            dataProvider={dataProvider}
            options={{ disableTelemetry: true, reactQuery: { clientConfig: client } }}
          >
            <RepositoryPipelinesTab
              repoId="31"
              defaultBranch="main"
              permissions={buildRepositoryPermissions("developer", false)}
              t={(key) => key}
              onError={() => {}}
            />
          </Refine>
        ),
      },
    ],
    { initialEntries: [path] },
  );
  render(<RouterProvider router={router} />);
};
describe("Empty pipelines with actual Refine result placeholders", () => {
  it("renders the canonical editor while the disabled detail query exposes Refine's initial empty object", async () => {
    setup("/app/projects/21/31/pipelines/editor");
    expect(await screen.findByRole("heading", { name: "Edit pipeline" })).toBeTruthy();
    expect(await screen.findByText("Pipeline canvas")).toBeTruthy();
    expect(screen.getByLabelText("Task name")).toHaveProperty("value", "test");
    expect(transport.mock.calls.some(([url]) => /\/pipelines\/0$/.test(url))).toBe(false);
    expect(screen.queryByText("Unexpected Application Error!")).toBeNull();
  });
  it("renders a genuinely empty pipeline list without treating the disabled detail placeholder as a pipeline", async () => {
    setup("/app/projects/21/31/pipelines");
    expect(await screen.findByText("No pipelines yet")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Edit configuration" })).toBeTruthy();
    expect(transport.mock.calls.map(([url]) => url)).toEqual(["/api/v1/projects/31/pipelines"]);
    expect(screen.queryByText("Unexpected Application Error!")).toBeNull();
  });
});
