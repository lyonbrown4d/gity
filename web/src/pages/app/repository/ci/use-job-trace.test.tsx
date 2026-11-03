// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useCustom } from "@refinedev/core";
import type { CiJob } from "@/lib/generated/model/ciJob";
import type { CiJobTrace } from "@/lib/generated/model/ciJobTrace";
import { tracePageSize, useJobTrace } from "./use-job-trace";
vi.mock("@refinedev/core", () => ({ useCustom: vi.fn() }));
const job: CiJob = {
  id: "41",
  project_id: "31",
  kind: "script",
  status: "failed",
  payload: "",
  result: "",
  attempts: 3,
  max_attempts: 3,
  run_after: "",
  locked_by: "",
  locked_until: "",
  last_error: "",
  started_at: "",
  finished_at: "",
  created_at: "",
  updated_at: "",
};
let response: CiJobTrace | undefined;
let requestError: Error | null;
beforeEach(() => {
  response = {
    job,
    attempts: [],
    log: null,
    output: "segment",
    attempt: 3,
    offset: 65537,
    next_offset: 131073,
    total_bytes: 200000,
    has_more: true,
    truncated: false,
    attempt_summaries: [],
  };
  requestError = null;
  vi.mocked(useCustom).mockReset();
  vi.mocked(useCustom).mockImplementation(
    () =>
      ({ result: { data: response }, query: { isFetching: false, error: requestError } }) as unknown as ReturnType<
        typeof useCustom
      >,
  );
});
afterEach(cleanup);
const query = () => vi.mocked(useCustom).mock.calls[vi.mocked(useCustom).mock.calls.length - 1]?.[0];
describe("Bounded job trace browsing", () => {
  it("requests a bounded page without retaining inactive page caches and uses server byte boundaries", () => {
    const view = renderHook(() => useJobTrace("31", job, true));
    expect(query()?.config?.query).toMatchObject({ limit: tracePageSize, offset: 0 });
    expect(query()?.queryOptions?.gcTime).toBe(0);
    act(() => view.result.current.next());
    expect(query()?.config?.query).toMatchObject({ offset: 131073 });
    expect(view.result.current.following).toBe(false);
    act(() => view.result.current.selectAttempt(2));
    expect(query()?.config?.query).toMatchObject({ attempt: 2, offset: 0 });
  });
  it("follows the latest segment only while requested and preserves the page when paused", () => {
    const view = renderHook(() => useJobTrace("31", job, true));
    act(() => view.result.current.latest());
    expect(query()?.config?.query).toMatchObject({ offset: 200000 - tracePageSize });
    act(() => view.result.current.setFollowing(false));
    response = { ...response!, total_bytes: 400000 };
    view.rerender();
    expect(query()?.config?.query).toMatchObject({ offset: 200000 - tracePageSize });
    expect(view.result.current.following).toBe(false);
  });
  it("retains the last received page on refresh failure and clears it when choosing another attempt", () => {
    const view = renderHook(() => useJobTrace("31", job, true));
    response = undefined;
    requestError = new Error("offline");
    view.rerender();
    expect(view.result.current.data?.output).toBe("segment");
    expect(view.result.current.query.error?.message).toBe("offline");
    act(() => view.result.current.selectAttempt(1));
    expect(view.result.current.data).toBeUndefined();
  });
});
