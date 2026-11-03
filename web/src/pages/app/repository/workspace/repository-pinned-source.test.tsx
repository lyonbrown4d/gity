import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { parsePinnedSourceTarget, RepositoryPinnedSource } from "./repository-pinned-source";
import type { RepositoryView } from "@/pages/types";

const { useCustom } = vi.hoisted(() => ({ useCustom: vi.fn() }));
vi.mock("@refinedev/core", () => ({ useCustom }));
vi.mock("./repository-source-canvas", () => ({ RepositorySourceCanvas: (props: { searchLine: number; selectedBlob: { content: string } }) => <div>{props.selectedBlob.content}:{props.searchLine}</div> }));
const commit = "a".repeat(40);
const repository = { id: "20", name: "Project" } as RepositoryView;
const props = { repository, target: { commit, path: "src/main.rs", line: 7 }, editorTheme: "light", t: (s: string) => s, onBrowse: vi.fn() };

describe("pinned global search source", () => {
  beforeEach(() => useCustom.mockReset());
  it("accepts a fixed commit and positive line, rejecting traversal or branch aliases", () => {
    expect(parsePinnedSourceTarget(`?commit_sha=${commit}&path=src%2Fmain.rs&line=7`)).toEqual(props.target);
    for (const value of [`?commit_sha=main&path=a&line=1`, `?commit_sha=${commit}&path=../secret&line=1`, `?commit_sha=${commit}&path=a&line=0`]) {
      expect(parsePinnedSourceTarget(value)).toBeNull();
    }
  });
  it("fetches the indexed revision and positions the existing source canvas", () => {
    useCustom.mockReturnValue({ query: { isSuccess: true, isFetching: false, isError: false }, result: { data: { content: "indexed source" } } });
    render(<RepositoryPinnedSource {...props} />);
    expect(screen.getByText("indexed source:7")).toBeTruthy();
    expect(useCustom).toHaveBeenCalledWith(expect.objectContaining({ config: { query: { ref_name: commit, path: props.target.path } } }));
  });
  it("hides cached source during authorization revalidation and on error", () => {
    useCustom.mockReturnValue({ query: { isSuccess: true, isFetching: true, isError: false }, result: { data: { content: "cached secret" } } });
    const view = render(<RepositoryPinnedSource {...props} />);
    expect(screen.queryByText(/cached secret/)).toBeNull();
    expect(screen.getByRole("status")).toBeTruthy();
    useCustom.mockReturnValue({ query: { isSuccess: false, isFetching: false, isError: true }, result: { data: { content: "cached secret" } } });
    view.rerender(<RepositoryPinnedSource {...props} />);
    expect(screen.queryByText(/cached secret/)).toBeNull();
    expect(screen.getByText("Unable to load file.")).toBeTruthy();
  });
});
