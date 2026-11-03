import { describe, expect, it } from "vitest";
import { mergeRequestRoute, parseMergeRequestIID, readMergeRequestFilters, updateMergeRequestFilters } from "./route-state";

describe("merge request route state", () => {
  it("accepts only positive numeric merge request IDs", () => {
    expect(parseMergeRequestIID("42")).toBe(42);
    expect(parseMergeRequestIID(undefined)).toBeNull();
    expect(parseMergeRequestIID("0")).toBeNull();
    expect(parseMergeRequestIID("42abc")).toBeNull();
  });

  it("keeps filters in the URL and omits default values", () => {
    const initial = new URLSearchParams("state=closed&q=fix&source=branch%3Arelease%2F1");
    expect(readMergeRequestFilters(initial)).toEqual({
      state: "closed", query: "fix", source: "branch:release/1", target: "all",
    });
    const cleared = updateMergeRequestFilters(initial, { state: "opened", query: "", source: "all" });
    expect(cleared.toString()).toBe("");
  });

  it("preserves encoded project path, selection, and list filters", () => {
    expect(mergeRequestRoute("org name", "123", 17, "?state=closed&q=fix"))
      .toBe("/app/projects/org%20name/123/merge-requests/17?state=closed&q=fix");
    expect(mergeRequestRoute("org", "123", undefined, "?state=closed&q=fix"))
      .toBe("/app/projects/org/123/merge-requests?state=closed&q=fix");
  });
});
