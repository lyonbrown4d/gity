// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, renderHook } from "@testing-library/react";
import { useCustom } from "@refinedev/core";
import { useGitignoreTemplates, useLicenseTemplates } from "./use-license-templates";

vi.mock("@refinedev/core", () => ({ useCustom: vi.fn() }));

describe("useLicenseTemplates", () => {
  beforeEach(() => {
    vi.mocked(useCustom).mockReset();
  });

  afterEach(cleanup);

  it("loads the public SPDX catalog through Refine and exposes an empty list while unavailable", () => {
    vi.mocked(useCustom).mockReturnValue({
      result: { data: [{ id: "mit", name: "MIT License", spdx_id: "MIT" }] },
      query: { isLoading: false, error: null, refetch: vi.fn() },
    } as never);

    const view = renderHook(() => useLicenseTemplates());

    expect(view.result.current.templates).toEqual([{ id: "mit", name: "MIT License", spdx_id: "MIT" }]);
    expect(vi.mocked(useCustom).mock.calls[0]?.[0]).toMatchObject({
      url: "/repository-templates/licenses",
      method: "get",
      meta: { auth: false },
    });
  });

  it("returns no selectable licenses and preserves the query error for retry UI", () => {
    const error = new Error("offline");
    vi.mocked(useCustom).mockReturnValue({
      result: { data: undefined },
      query: { isLoading: false, error, refetch: vi.fn() },
    } as never);

    const view = renderHook(() => useLicenseTemplates());

    expect(view.result.current.templates).toEqual([]);
    expect(view.result.current.query.error).toBe(error);
  });

  it("loads gitignore templates independently from the license catalog", () => {
    vi.mocked(useCustom).mockImplementation((options) => {
      const isGitignore = options.url === "/repository-templates/gitignores";
      return {
        result: { data: isGitignore ? [{ id: "node", name: "Node" }] : undefined },
        query: { isLoading: false, error: isGitignore ? null : new Error("license offline"), refetch: vi.fn() },
      } as never;
    });

    const gitignores = renderHook(() => useGitignoreTemplates());
    const licenses = renderHook(() => useLicenseTemplates());

    expect(gitignores.result.current.templates).toEqual([{ id: "node", name: "Node" }]);
    expect(gitignores.result.current.query.error).toBeNull();
    expect(licenses.result.current.templates).toEqual([]);
    expect(licenses.result.current.query.error).toEqual(new Error("license offline"));
    expect(vi.mocked(useCustom).mock.calls.map(([options]) => options.url)).toEqual([
      "/repository-templates/gitignores",
      "/repository-templates/licenses",
    ]);
  });
});
