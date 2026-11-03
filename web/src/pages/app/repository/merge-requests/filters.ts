import type { RepositoryMergeRequestState, RepositoryMergeRequestView } from "@/pages/types";
import { toTimestamp } from "../issues-utils";

export type MergeRequestStateFilter = RepositoryMergeRequestState | "all";
export type MergeRequestBranchFilter = "all" | `branch:${string}`;
export interface MergeRequestBranchFilterOption { name: string; count: number; }
export interface MergeRequestFilterCriteria {
  state: MergeRequestStateFilter;
  query: string;
  sourceBranch: MergeRequestBranchFilter;
  targetBranch: MergeRequestBranchFilter;
}

export const mergeRequestBranchFilterValue = (branch: string): MergeRequestBranchFilter => `branch:${branch}`;

const selectedMergeRequestBranch = (filter: MergeRequestBranchFilter): string => (
  filter.startsWith("branch:") ? filter.slice("branch:".length) : ""
);

export const buildMergeRequestBranchFilterOptions = (
  items: RepositoryMergeRequestView[],
  branchField: "source_branch" | "target_branch",
): MergeRequestBranchFilterOption[] => {
  const counts = new Map<string, number>();
  for (const item of items) {
    const branch = item[branchField].trim();
    if (branch) {
      counts.set(branch, (counts.get(branch) ?? 0) + 1);
    }
  }
  return Array.from(counts, ([name, count]) => ({ name, count }))
    .sort((left, right) => left.name.localeCompare(right.name));
};

export const filterMergeRequests = (
  items: RepositoryMergeRequestView[],
  criteria: MergeRequestFilterCriteria,
): RepositoryMergeRequestView[] => {
  const normalizedQuery = criteria.query.trim().toLowerCase();
  return items
    .filter((item) => criteria.state === "all" || item.state === criteria.state)
    .filter((item) => mergeRequestMatchesBranchFilter(item.source_branch, criteria.sourceBranch))
    .filter((item) => mergeRequestMatchesBranchFilter(item.target_branch, criteria.targetBranch))
    .filter((item) => {
      if (!normalizedQuery) {
        return true;
      }
      return `${item.iid} ${item.title} ${item.description ?? ""} ${item.source_branch} ${item.target_branch}`
        .toLowerCase()
        .includes(normalizedQuery);
    })
    .sort((a, b) => {
      const left = toTimestamp(a.updated_at ?? "");
      const right = toTimestamp(b.updated_at ?? "");
      return (Number.isFinite(right) ? right : 0) - (Number.isFinite(left) ? left : 0);
    });
};

const mergeRequestMatchesBranchFilter = (branch: string, filter: MergeRequestBranchFilter): boolean => {
  if (filter === "all") {
    return true;
  }
  return branch === selectedMergeRequestBranch(filter);
};
