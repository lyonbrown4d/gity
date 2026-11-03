import { expect, it } from "vitest";
import { normalizeMergeRequest, resolveMergeRequestList } from "./data";

it("reads a real page envelope instead of displaying it as an empty list", () => {
  const row = {
    id: "375848329264762881",
    iid: 23,
    title: "Review access",
    state: "opened",
    source_branch: "feature",
    target_branch: "main",
  };
  const items = resolveMergeRequestList({ items: [row], page: 1, page_size: 20, total: 1, total_pages: 1 });
  expect(items.map(normalizeMergeRequest)).toMatchObject([{ id: row.id, iid: 23, title: row.title }]);
  expect(resolveMergeRequestList({ items: [], total: 0 })).toEqual([]);
});
