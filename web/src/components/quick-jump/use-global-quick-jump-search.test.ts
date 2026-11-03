import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SearchReply } from "@/lib/generated/model";
const { useCustom } = vi.hoisted(() => ({ useCustom: vi.fn() }));
vi.mock("@refinedev/core", () => ({ useCustom }));
import { useGlobalQuickJumpSearch } from "./use-global-quick-jump-search";

const result: SearchReply = {
  groups: [{ kind: "issue", items: [{ kind: "issue", organization_id: "3", project_id: "1", project_path: "team/app", resource_id: "2", title: "needle", snippet: "text", path: null, commit_sha: null, line: null, issue_iid: "2", state: "open" }] }],
  next_cursor: "next-page",
  partial: true,
  indexing_eventually_consistent: true,
  max_code_files_per_project: 100,
  max_code_file_size_bytes: 1000,
};

beforeEach(() => {
  vi.useFakeTimers();
  useCustom.mockReset();
  useCustom.mockReturnValue({
    query: { isSuccess: false, isFetching: false, isError: false, error: null },
    result: { data: undefined },
  });
});

describe("global quick jump search state", () => {
  it("waits for the current query and returns grouped results and partial status", async () => {
    const hook = renderHook(({ query }) => useGlobalQuickJumpSearch(query, true), { initialProps: { query: "" } });
    expect(hook.result.current.isTooShort).toBe(true);
    hook.rerender({ query: "needle" });
    expect(hook.result.current.items).toEqual([]);
    expect(hook.result.current.isSearching).toBe(true);
    useCustom.mockReturnValue({
      query: { isSuccess: true, isFetching: false, isError: false, error: null },
      result: { data: result },
    });
    await act(async () => { vi.advanceTimersByTime(250); });
    expect(hook.result.current.items).toHaveLength(1);
    expect(hook.result.current.partial).toBe(true);
    expect(hook.result.current.hasMore).toBe(true);
    const lastCall = useCustom.mock.calls[useCustom.mock.calls.length - 1];
    expect(lastCall?.[0]).toMatchObject({
      url: "/search",
      method: "post",
      config: { payload: { query: "needle", types: ["code", "issue"], limit: 8 } },
    });
  });

  it("hides cached results while the query changes and after permission errors", async () => {
    useCustom.mockReturnValue({
      query: { isSuccess: true, isFetching: false, isError: false, error: null },
      result: { data: result },
    });
    const hook = renderHook(({ query }) => useGlobalQuickJumpSearch(query, true), { initialProps: { query: "needle" } });
    await act(async () => { vi.advanceTimersByTime(250); });
    expect(hook.result.current.items).toHaveLength(1);
    hook.rerender({ query: "restricted" });
    expect(hook.result.current.items).toEqual([]);
    useCustom.mockReturnValue({
      query: { isSuccess: false, isFetching: false, isError: true, error: new Error("Forbidden") },
      result: { data: result },
    });
    await act(async () => { vi.advanceTimersByTime(250); });
    expect(hook.result.current.items).toEqual([]);
    expect(hook.result.current.error).toMatchObject({ message: "Forbidden" });
    expect(hook.result.current.hasMore).toBe(false);
  });
});
