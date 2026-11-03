import { describe, expect, it } from "vitest";
import type { RepositoryBranchProtectionView } from "@/pages/types";
import { buildRepositoryPermissions } from "../repository-permissions";
import { branchWriteBlock } from "./branch-access";
const rule: RepositoryBranchProtectionView = {
  id: "1",
  repository_id: "1",
  branch_name: "main",
  rule_type: "exact",
  push_access_level: "maintainer",
  merge_access_level: "maintainer",
  require_merge_request: false,
  require_pipeline_success: false,
  allow_delete: false,
  allow_force_push: false,
};
describe("CI branch commit preconditions", () => {
  it("allows eligible writers while blocking read-only roles, protected role mismatches and MR-only branches", () => {
    const developer = buildRepositoryPermissions("developer", false);
    expect(branchWriteBlock(developer)).toBeNull();
    expect(branchWriteBlock(buildRepositoryPermissions("reporter", false))).toContain("write permission");
    expect(branchWriteBlock(developer, rule)).toContain("cannot push");
    expect(branchWriteBlock(buildRepositoryPermissions("maintainer", false), rule)).toBeNull();
    expect(
      branchWriteBlock(buildRepositoryPermissions("owner", true), { ...rule, require_merge_request: true }),
    ).toContain("merge request");
    expect(
      branchWriteBlock(buildRepositoryPermissions("owner", true), { ...rule, push_access_level: "no_one" }),
    ).toContain("cannot push");
  });
});
