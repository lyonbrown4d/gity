import { cleanup, render, screen, within } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes, useLocation } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { RepositoryView } from "@/pages/types";
import type { TodoView } from "../todo-types";
import { AppDashboardPage } from "../dashboard-page";

const state = vi.hoisted(() => ({
  repositories: [] as RepositoryView[],
  todos: [] as TodoView[],
  todoError: null as Error | null,
  repoError: null as Error | null,
}));

vi.mock("@refinedev/core", () => ({
  useOne: () => ({ result: { username: "dev" }, query: { isLoading: false, error: null } }),
  useList: () => ({
    result: { data: state.repositories },
    query: { isLoading: false, error: state.repoError, refetch: vi.fn() },
  }),
}));

vi.mock("@/lib/i18n", () => ({ useI18n: () => ({ t: (value: string) => value }) }));
vi.mock("../use-todos", () => ({
  useTodoList: () => ({
    todos: state.todos,
    query: { isLoading: false, error: state.todoError, refetch: vi.fn() },
  }),
}));
vi.mock("../use-activity", () => ({
  useActivity: () => ({ items: [], query: { isLoading: false, error: null } }),
}));

const repositories: RepositoryView[] = [
  {
    id: "17", uuid: "uuid-17", organization_id: "4", key: "alpha", full_path: "team/alpha",
    name: "Alpha repository", description: "Frontend services", visibility: "private",
    default_branch: "main", status: "active", clone_http_url: "https://example.test/alpha",
  },
  {
    id: "18", uuid: "uuid-18", organization_id: "4", key: "beta", full_path: "team/beta",
    name: "Beta repository", description: "Data tools", visibility: "private",
    default_branch: "trunk", status: "active", clone_http_url: "https://example.test/beta",
  },
];

const todos: TodoView[] = [
  {
    id: "71", user_id: "8", organization_id: "4", project_id: "17", kind: "merge_request_review_requested",
    state: "pending", target_type: "merge_request", target_id: "128", title: "Review access checks",
    summary: "Please review the permission changes", action_url: "/app/projects/4/17/merge-requests/128",
    created_at: "2026-11-03T10:24:00Z", updated_at: "2026-11-03T10:24:00Z",
  },
  {
    id: "72", user_id: "8", organization_id: "4", project_id: "18", kind: "issue_assigned",
    state: "pending", target_type: "issue", target_id: "31", title: "Document the new API",
    summary: "Update the integration guide", action_url: "/app/projects/4/18/issues/31",
    created_at: "2026-11-02T08:00:00Z", updated_at: "2026-11-02T08:00:00Z",
  },
];

const LocationProbe = (): JSX.Element => {
  const location = useLocation();
  return <output aria-label="Current location">{location.pathname}</output>;
};

const renderPage = (initialEntry = "/app/dashboard"): ReturnType<typeof render> => render(<MemoryRouter initialEntries={[initialEntry]}><Routes>
    <Route path="/app/dashboard" element={<AppDashboardPage />} />
    <Route path="*" element={<LocationProbe />} />
  </Routes></MemoryRouter>);

describe("dashboard workbench", () => {
  afterEach(cleanup);

  beforeEach(() => {
    state.repositories = repositories;
    state.todos = todos;
    state.todoError = null;
    state.repoError = null;
  });

  it("filters to review requests, shows the selected item, and opens its real action route", async () => {
    const user = userEvent.setup();
    renderPage();

    expect(screen.getByRole("heading", { name: "Recent projects" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Recent activity" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /New Project/ })).toHaveAttribute("href", "/app/projects?create=1");
    await user.click(screen.getByRole("tab", { name: /Review requests/ }));

    expect(screen.getAllByText("Review access checks")).toHaveLength(2);
    expect(screen.queryByText("Document the new API")).not.toBeInTheDocument();
    expect(screen.getAllByText("Please review the permission changes")).toHaveLength(2);
    expect(screen.getAllByText("Alpha repository")).toHaveLength(2);
    await user.click(screen.getByRole("link", { name: "Open item" }));
    expect(screen.getByLabelText("Current location")).toHaveTextContent("/app/projects/4/17/merge-requests/128");
  });

  it("selects a work row and updates the contextual details", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByText("Document the new API"));

    const panel = screen.getByLabelText("Work details");
    expect(within(panel).getByText("Update the integration guide")).toBeInTheDocument();
    expect(within(panel).getByText("Beta repository")).toBeInTheDocument();
    expect(within(panel).getByRole("link", { name: "Open item" })).toHaveAttribute("href", "/app/projects/4/18/issues/31");
  });

  it("keeps adjacent snowflake IDs distinct when selecting work", async () => {
    const user = userEvent.setup();
    state.todos = [
      { ...todos[0], id: "9007199254740992", title: "First adjacent snowflake", summary: "First exact ID" },
      { ...todos[1], id: "9007199254740993", title: "Second adjacent snowflake", summary: "Second exact ID" },
    ];
    renderPage();

    await user.click(screen.getByText("Second adjacent snowflake"));

    const panel = screen.getByLabelText("Work details");
    expect(within(panel).getByText("Second adjacent snowflake")).toBeInTheDocument();
    expect(within(panel).getByText("Second exact ID")).toBeInTheDocument();
    expect(within(panel).queryByText("First exact ID")).not.toBeInTheDocument();
  });

  it("keeps request failures distinct from a successful empty state", () => {
    state.todos = [];
    state.todoError = new Error("Request timed out");
    renderPage();

    expect(screen.getByRole("alert")).toHaveTextContent("Request timed out");
    expect(screen.getByText("Work is unavailable until the request succeeds.")).toBeInTheDocument();
    expect(screen.queryByText("No pending work in this view.")).not.toBeInTheDocument();
  });

  it("searches recent projects by repository name from the default work view", async () => {
    const user = userEvent.setup();
    renderPage();

    await user.type(screen.getByRole("textbox", { name: "Search your work and projects" }), "Beta");

    expect(screen.getByRole("link", { name: /Beta repository/ })).toHaveAttribute("href", "/app/projects/4/18");
    expect(screen.queryByRole("link", { name: /Alpha repository/ })).not.toBeInTheDocument();
  });
});
