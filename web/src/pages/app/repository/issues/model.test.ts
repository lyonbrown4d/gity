import { describe, expect, it } from "vitest";
import type { RepositoryIssueView } from "@/pages/types";
import { filterAndSortIssuesForTriage } from "./model";

const issue = (patch: Partial<RepositoryIssueView>): RepositoryIssueView => ({
  id: "1", repository_id: "9", number: 1, title: "Update docs", description: "guide",
  status: "open", author_user_id: "12", assignee_user_ids: [], labels: [],
  created_at: "2026-01-01T00:00:00Z", updated_at: "2026-01-01T00:00:00Z", ...patch,
});

describe("repository issue triage filters", () => {
  it("combines status, label, assignee and case-insensitive search", () => {
    const items = [
      issue({ id: "1", number: 1, title: "Fix Search", labels: [{ id: "1", issue_id: "1", name: "bug", created_at: "", updated_at: "" }], assignee_user_ids: ["7"] }),
      issue({ id: "2", number: 2, status: "closed", title: "Search docs" }),
    ];
    const result = filterAndSortIssuesForTriage(items, {
      status: "open", label: "label:bug", assignee: "user:7", query: " SEARCH ", sort: "number_desc", userByID: new Map(),
    });
    expect(result.map((item) => item.id)).toEqual(["1"]);
  });

  it("sorts by newest issue number and filters unassigned issues", () => {
    const items = [issue({ id: "1", number: 3 }), issue({ id: "2", number: 8, assignee_user_ids: ["7"] })];
    const result = filterAndSortIssuesForTriage(items, {
      status: "all", label: "all", assignee: "unassigned", query: "", sort: "number_desc", userByID: new Map(),
    });
    expect(result.map((item) => item.number)).toEqual([3]);
  });
});
