import type { RepositoryBranchProtectionView } from "@/pages/types";
import type { RepositoryPermissions } from "../repository-permissions";
/** Mirrors the direct-push preconditions; the server remains authoritative. */
export const branchWriteBlock = (
  permissions: RepositoryPermissions,
  protection?: RepositoryBranchProtectionView | null,
): string | null => {
  if (!permissions.repositoryPush) return "Repository write permission is required to commit configuration.";
  if (!protection) return null;
  if (protection.require_merge_request)
    return "This protected branch requires a merge request. Select a writable branch.";
  const levels: Record<string, number> = { guest: 10, reporter: 20, developer: 30, maintainer: 40, owner: 50 };
  const actor = permissions.isSuperAdmin ? 100 : (levels[permissions.role] ?? 0);
  const minimum = levels[protection.push_access_level] ?? Infinity;
  return actor < minimum ? "Your role cannot push to this protected branch. Select a writable branch." : null;
};
