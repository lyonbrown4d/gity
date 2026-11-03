import type { Issue, User } from "@/lib/generated/model";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import { formatUserLabel } from "../repository-user-utils";
import { toTimestamp, type IssueSortMode } from "../issues-utils";

export type IssueStatusFilter = RepositoryIssueView["status"] | "all";
export type IssueLabelFilter = "all" | "unlabeled" | `label:${string}`;
export type IssueAssigneeFilter = "all" | "unassigned" | `user:${string}`;

export interface IssueLabelFilterOption {
  name: string;
  count: number;
}

export interface IssueAssigneeFilterOption {
  id: string;
  label: string;
  count: number;
}

export interface RepositoryIssueStats {
  total: number;
  open: number;
  closed: number;
  unassigned: number;
  labeled: number;
}

export interface IssueFilterCriteria {
  status: IssueStatusFilter;
  label: IssueLabelFilter;
  assignee: IssueAssigneeFilter;
  query: string;
  sort: IssueSortMode;
  userByID: Map<string, UserView>;
}

export const issueAssigneeIDs = (issue: RepositoryIssueView): string[] => {
  const values = issue.assignee_user_ids?.length
    ? issue.assignee_user_ids
    : issue.assignee_user_id ? [issue.assignee_user_id] : [];
  return values.map(String).filter(Boolean);
};

export const issueLabelFilterValue = (name: string): IssueLabelFilter => `label:${name}`;
export const issueAssigneeFilterValue = (userID: string): IssueAssigneeFilter => `user:${userID}`;

const selectedIssueLabelName = (filter: IssueLabelFilter): string =>
  filter.startsWith("label:") ? filter.slice("label:".length) : "";

const selectedIssueAssigneeID = (filter: IssueAssigneeFilter): string =>
  filter.startsWith("user:") ? filter.slice("user:".length) : "";

export const buildIssueLabelFilterOptions = (issues: RepositoryIssueView[]): IssueLabelFilterOption[] => {
  const counts = new Map<string, number>();
  for (const issue of issues) {
    for (const label of issue.labels ?? []) {
      const name = label.name.trim();
      if (name) counts.set(name, (counts.get(name) ?? 0) + 1);
    }
  }
  return Array.from(counts, ([name, count]) => ({ name, count }))
    .sort((left, right) => left.name.localeCompare(right.name));
};

export const buildIssueAssigneeFilterOptions = (
  issues: RepositoryIssueView[],
  userByID: Map<string, UserView>,
): IssueAssigneeFilterOption[] => {
  const counts = new Map<string, number>();
  for (const issue of issues) {
    for (const userID of issueAssigneeIDs(issue)) counts.set(userID, (counts.get(userID) ?? 0) + 1);
  }
  return Array.from(counts, ([id, count]) => ({
    id, count, label: formatUserLabel(userByID.get(id), id),
  })).sort((left, right) => left.label.localeCompare(right.label));
};

export const filterAndSortIssuesForTriage = (
  items: RepositoryIssueView[],
  criteria: IssueFilterCriteria,
): RepositoryIssueView[] => {
  const normalizedQuery = normalizeIssueSearch(criteria.query);
  const filtered = items.filter((item) => {
    if (criteria.status !== "all" && item.status !== criteria.status) return false;
    if (!issueMatchesLabelFilter(item, criteria.label)) return false;
    if (!issueMatchesAssigneeFilter(item, criteria.assignee)) return false;
    return !normalizedQuery || issueMatchesSearch(item, normalizedQuery, criteria.userByID);
  });
  return [...filtered].sort((left, right) => {
    if (criteria.sort === "updated_desc") return toTimestamp(right.updated_at) - toTimestamp(left.updated_at);
    if (criteria.sort === "created_desc") return toTimestamp(right.created_at) - toTimestamp(left.created_at);
    if (criteria.sort === "number_asc") return left.number - right.number;
    return right.number - left.number;
  });
};

const issueMatchesLabelFilter = (issue: RepositoryIssueView, filter: IssueLabelFilter): boolean => {
  if (filter === "all") return true;
  const labels = (issue.labels ?? []).map((label) => label.name.trim()).filter(Boolean);
  if (filter === "unlabeled") return labels.length === 0;
  return labels.includes(selectedIssueLabelName(filter));
};

const issueMatchesAssigneeFilter = (issue: RepositoryIssueView, filter: IssueAssigneeFilter): boolean => {
  if (filter === "all") return true;
  const assigneeIDs = issueAssigneeIDs(issue);
  if (filter === "unassigned") return assigneeIDs.length === 0;
  return assigneeIDs.includes(selectedIssueAssigneeID(filter));
};

const issueMatchesSearch = (
  issue: RepositoryIssueView,
  normalizedQuery: string,
  userByID: Map<string, UserView>,
): boolean => {
  const assigneeIDs = issueAssigneeIDs(issue);
  const searchableValues = [
    `#${issue.number}`, String(issue.number), issue.title, issue.description ?? "", issue.author_user_id,
    formatUserLabel(userByID.get(issue.author_user_id), issue.author_user_id),
    ...(issue.labels ?? []).map((label) => label.name),
    ...assigneeIDs.flatMap((userID) => [userID, formatUserLabel(userByID.get(userID), userID)]),
  ];
  return searchableValues.some((value) => normalizeIssueSearch(value).includes(normalizedQuery));
};

const normalizeIssueSearch = (value: string): string => value.trim().toLowerCase();

/** Orval models issue status as a string; the view uses the known status union. */
export const toIssueView = (issue: Issue): RepositoryIssueView => ({
  id: issue.id,
  repository_id: issue.repository_id,
  number: issue.number,
  title: issue.title,
  description: issue.description,
  status: issue.status === "closed" ? "closed" : "open",
  author_user_id: issue.author_user_id,
  assignee_user_id: issue.assignee_user_id ?? null,
  assignee_user_ids: issue.assignee_user_ids,
  labels: issue.labels.map((label) => ({
    id: label.id,
    issue_id: label.issue_id,
    name: label.name,
    color: label.color,
    created_at: label.created_at,
    updated_at: label.updated_at,
  })),
  created_at: issue.created_at,
  updated_at: issue.updated_at,
  closed_at: issue.closed_at ?? null,
});

export const toUserView = (user: User): UserView => ({
  id: user.id,
  username: user.username,
  display_name: user.display_name || null,
  email: user.email ?? "",
  status: user.status,
  is_super_admin: user.is_super_admin,
});
