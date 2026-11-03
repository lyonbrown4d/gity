import { describe, expect, it } from "vitest";
import { parseDiff, resolveInlineReviewThreads } from "./diff-review";

describe("merge request diff review", () => {
  it("maps hunk lines to the changed file and target line numbers", () => {
    const lines = parseDiff([
      "diff --git a/src/lib.rs b/src/lib.rs",
      "--- a/src/lib.rs",
      "+++ b/src/lib.rs",
      "@@ -10,2 +10,3 @@",
      " keep",
      "-old",
      "+new",
      "+next",
    ].join("\n"));

    expect(lines.filter((line) => line.kind !== "meta")).toEqual([
      { text: " keep", kind: "context", file: "src/lib.rs", line: 10 },
      { text: "-old", kind: "removed", file: "src/lib.rs", line: 11 },
      { text: "+new", kind: "added", file: "src/lib.rs", line: 11 },
      { text: "+next", kind: "added", file: "src/lib.rs", line: 12 },
    ]);
  });

  it("normalizes inline discussion anchors and ignores malformed positions", () => {
    const threads = resolveInlineReviewThreads({ body: [{
      id: "31",
      resolved: false,
      comments: [
        { id: "32", author_user_id: "8", body: "Check this", position: { file: "src/lib.rs", line: 14, column: 3 } },
        { id: "33", author_user_id: "9", body: "Plain note", position: null },
      ],
    }] });

    expect(threads).toHaveLength(1);
    expect(threads[0].comments[0].position).toEqual({ file: "src/lib.rs", line: 14, column: 3 });
    expect(threads[0].comments[1].position).toBeNull();
  });
});
