// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useCIDraft } from "./use-ci-draft";
const key = "gity.ci-draft:test:main";
const baseline = { source: "jobs: {}", original: "jobs: {}", head: "original-head", exists: true };
afterEach(() => {
  cleanup();
  sessionStorage.clear();
});
describe("CI draft recovery", () => {
  it("keeps unsaved YAML per browser tab and clears the record after saving", () => {
    const view = renderHook(({ source, original }) => useCIDraft(key, true, { ...baseline, source, original }), {
      initialProps: { source: baseline.source, original: baseline.original },
    });
    view.rerender({ source: "jobs: {changed: true}", original: baseline.original });
    expect(JSON.parse(sessionStorage.getItem(key)!).source).toContain("changed");
    view.rerender({ source: "jobs: {changed: true}", original: "jobs: {changed: true}" });
    expect(sessionStorage.getItem(key)).toBeNull();
  });
  it("offers the old draft without overwriting it with a newer branch snapshot", () => {
    const saved = { ...baseline, source: "unsaved YAML" };
    sessionStorage.setItem(key, JSON.stringify(saved));
    const view = renderHook(() =>
      useCIDraft(key, true, {
        ...baseline,
        source: "repository update",
        original: "repository update",
        head: "new-head",
      }),
    );
    expect(view.result.current.recoverable).toEqual(saved);
    expect(view.result.current.recoverable?.head).toBe("original-head");
    expect(JSON.parse(sessionStorage.getItem(key)!).source).toBe("unsaved YAML");
    act(() => {
      view.result.current.discard();
      view.result.current.resolve();
    });
    expect(view.result.current.recoverable).toBeNull();
    expect(sessionStorage.getItem(key)).toBeNull();
  });
});
