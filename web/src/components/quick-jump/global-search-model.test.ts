import { describe, expect, it } from "vitest";
import { toGlobalSearchQuickJumpItem } from "./global-search-model";
import type { SearchHit } from "@/lib/generated/model";
const hit = (patch: Partial<SearchHit>): SearchHit => ({
  kind: "issue",
  organization_id: "12",
  project_id: "9223372036854775806",
  project_path: "team/platform",
  resource_id: "issue-row-id",
  title: "Private regression",
  snippet: "short safe excerpt",
  path: null,
  commit_sha: null,
  line: null,
  issue_iid: "7",
  state: "open",
  ...patch,
});

describe("global quick jump result destinations", () => {
  it("opens issue results using the issue IID and project route identifiers", () => {
    const item = toGlobalSearchQuickJumpItem(hit({}));
    expect(item?.path).toBe("/app/projects/12/9223372036854775806/issues/7");
    expect(item?.description).toContain("team/platform · #7");
  });

  it("pins code links to the indexed commit and line", () => {
    const item = toGlobalSearchQuickJumpItem(hit({
      kind: "code",
      path: "src/main.rs",
      commit_sha: "0123456789abcdef",
      line: 42,
      issue_iid: null,
      state: null,
    }));
    expect(item?.path).toContain("commit_sha=0123456789abcdef");
    expect(item?.path).toContain("path=src%2Fmain.rs");
    expect(item?.path).toContain("line=42#L42");
  });

  it("omits malformed code hits without an immutable revision and line", () => {
    expect(toGlobalSearchQuickJumpItem(hit({ kind: "code" }))).toBeNull();
  });
});
