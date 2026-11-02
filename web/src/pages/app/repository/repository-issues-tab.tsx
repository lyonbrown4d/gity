import { useEffect, useMemo, useState } from "react";
import { CircleDot, Search, Tags, UserRound } from "lucide-react";
import { useNavigate } from "react-router";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  useGetApiV1ProjectsIdIssues,
  usePatchApiV1ProjectsIdIssuesIidAssignees,
  usePostApiV1ProjectsIdIssues,
} from "@/lib/generated/issues/issues";
import { useGetApiV1Users } from "@/lib/generated/users/users";
import type { Issue, User } from "@/lib/generated/model";
import { toApiPathId, toRawJsonId } from "@/lib/api-id";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import { buildIssueDetailPath } from "./issue-markdown";
import { IssueMarkdownEditor } from "./issue-markdown-editor";
import { extractErrorMessage, formatRelativeTime, toTimestamp, type IssueSortMode } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { formatUserLabel } from "./repository-user-utils";

interface RepositoryIssuesTabProps {
  organizationId: string;
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type IssueStatusFilter = RepositoryIssueView["status"] | "all";
type IssueLabelFilter = "all" | "unlabeled" | `label:${string}`;
type IssueAssigneeFilter = "all" | "unassigned" | `user:${string}`;

interface IssueLabelFilterOption {
  name: string;
  count: number;
}

interface IssueAssigneeFilterOption {
  id: string;
  label: string;
  count: number;
}

interface IssueFilterCriteria {
  status: IssueStatusFilter;
  label: IssueLabelFilter;
  assignee: IssueAssigneeFilter;
  query: string;
  sort: IssueSortMode;
  userByID: Map<string, UserView>;
}

export const RepositoryIssuesTab = ({
  organizationId,
  repoId,
  permissions,
  t,
  onError,
}: RepositoryIssuesTabProps): JSX.Element => {
  const navigate = useNavigate();
  const apiProjectId = toApiPathId(repoId);
  const issuesQuery = useGetApiV1ProjectsIdIssues(
    apiProjectId,
    { state: "all", page: 1, page_size: 100 },
    {
      query: {
        enabled: Boolean(repoId),
        refetchOnWindowFocus: false,
      },
    },
  );
  const usersQuery = useGetApiV1Users(
    { page: 1, page_size: 100 },
    {
      query: {
        enabled: Boolean(repoId),
        refetchOnWindowFocus: false,
      },
    },
  );
  const createIssueMutation = usePostApiV1ProjectsIdIssues();
  const setAssigneesMutation = usePatchApiV1ProjectsIdIssuesIidAssignees();
  const issues = useMemo<RepositoryIssueView[]>(
    () => (issuesQuery.data?.data.items ?? []).map(toIssueView),
    [issuesQuery.data],
  );
  const users = useMemo(
    () => (usersQuery.data?.data.items ?? []).map(toUserView),
    [usersQuery.data],
  );
  const userByID = useMemo(
    () => new Map(users.map((user) => [user.id, user])),
    [users],
  );
  const [issueStatusFilter, setIssueStatusFilter] = useState<IssueStatusFilter>("open");
  const [issueSearchQuery, setIssueSearchQuery] = useState("");
  const [issueLabelFilter, setIssueLabelFilter] = useState<IssueLabelFilter>("all");
  const [issueAssigneeFilter, setIssueAssigneeFilter] = useState<IssueAssigneeFilter>("all");
  const [issueSort, setIssueSort] = useState<IssueSortMode>("updated_desc");
  const [isIssueComposerOpen, setIssueComposerOpen] = useState(false);
  const [newIssueTitle, setNewIssueTitle] = useState("");
  const [newIssueDescription, setNewIssueDescription] = useState("");
  const [newIssueAssigneeId, setNewIssueAssigneeId] = useState("");

  const issueStats = useMemo(
    () => ({
      total: issues.length,
      open: issues.filter((item) => item.status === "open").length,
      closed: issues.filter((item) => item.status === "closed").length,
      unassigned: issues.filter((item) => issueAssigneeIDs(item).length === 0).length,
      labeled: issues.filter((item) => (item.labels ?? []).length > 0).length,
    }),
    [issues],
  );
  const issueLabelOptions = useMemo(
    () => buildIssueLabelFilterOptions(issues),
    [issues],
  );
  const issueAssigneeOptions = useMemo(
    () => buildIssueAssigneeFilterOptions(issues, userByID),
    [issues, userByID],
  );
  const filteredIssues = useMemo(
    () => filterAndSortIssuesForTriage(issues, {
      status: issueStatusFilter,
      label: issueLabelFilter,
      assignee: issueAssigneeFilter,
      query: issueSearchQuery,
      sort: issueSort,
      userByID,
    }),
    [issues, issueStatusFilter, issueLabelFilter, issueAssigneeFilter, issueSearchQuery, issueSort, userByID],
  );
  const isLoadingIssues = issuesQuery.isFetching && !issuesQuery.data;
  const isCreatingIssue = createIssueMutation.isPending;
  const canCreateIssue = permissions.issueCreate;
  const hasIssueFilters = issueStatusFilter !== "open"
    || issueLabelFilter !== "all"
    || issueAssigneeFilter !== "all"
    || issueSearchQuery.trim().length > 0;
  const resetIssueFilters = () => {
    setIssueStatusFilter("open");
    setIssueSearchQuery("");
    setIssueLabelFilter("all");
    setIssueAssigneeFilter("all");
    setIssueSort("updated_desc");
  };

  const loadIssues = async () => {
    const result = await issuesQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitCreateIssue = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const title = newIssueTitle.trim();
    if (!title) {
      onError(t("Issue title is required"));
      return;
    }

    onError(null);
    try {
      const response = await createIssueMutation.mutateAsync({
        id: apiProjectId,
        data: {
          title,
          description: newIssueDescription.trim(),
        },
      });
      const assigneeID = newIssueAssigneeId.trim();
      if (assigneeID) {
        await setAssigneesMutation.mutateAsync({
          id: apiProjectId,
          iid: response.data.number,
          data: {
            // `UserIds.user_ids` is generated as `number[]` because the spec
            // models it as int64. `Number()` rounds snowflake IDs past 2^53 and
            // the backend rejects decimal strings, so emit exact JSON numbers.
            user_ids: [toRawJsonId(assigneeID)],
          },
        });
      }
      setNewIssueTitle("");
      setNewIssueDescription("");
      setNewIssueAssigneeId("");
      setIssueComposerOpen(false);
      await loadIssues();
      navigate(buildIssueDetailPath(organizationId, repoId, response.data.number));
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
  }, [repoId, onError]);

  useEffect(() => {
    const error = issuesQuery.error ?? usersQuery.error;
    if (!error) {
      return;
    }
    onError(extractErrorMessage(error));
  }, [issuesQuery.error, usersQuery.error, onError]);

  return (
    <Section
      title={t("Issues")}
      description={t("Track bugs, tasks, ownership, labels, and discussions for this project.")}
      actions={
        <>
          <Button
            type="button"
            size="sm"
            variant={isIssueComposerOpen ? "secondary" : "outline"}
            disabled={!canCreateIssue}
            onClick={() => setIssueComposerOpen((current) => !current)}
          >
            {isIssueComposerOpen ? t("Hide new issue form") : t("New issue")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadIssues()}>
            {t("Reload")}
          </Button>
        </>
      }
      flush
      className="card-enter overflow-hidden"
    >
      <div className="grid divide-y divide-border border-b border-border sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-5">
        <IssueStat label={t("Open")} value={issueStats.open} />
        <IssueStat label={t("Closed")} value={issueStats.closed} />
        <IssueStat label={t("Unassigned")} value={issueStats.unassigned} />
        <IssueStat label={t("Labeled")} value={issueStats.labeled} />
        <IssueStat label={t("Total")} value={issueStats.total} />
      </div>

      <div className="border-b border-border px-4 py-3">
        <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_160px_180px_180px_170px]">
          <div className="relative md:col-span-2 xl:col-span-1">
            <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="h-8 pl-8"
              placeholder={t("Search title, label, or assignee")}
              value={issueSearchQuery}
              onChange={(event) => setIssueSearchQuery(event.target.value)}
            />
          </div>
          <Select
            value={issueStatusFilter}
            onValueChange={(value) => setIssueStatusFilter(value as IssueStatusFilter)}
          >
            <SelectTrigger className="h-8">
              <SelectValue placeholder={t("Status")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="open">{t("Open issues")}</SelectItem>
                <SelectItem value="closed">{t("Closed issues")}</SelectItem>
                <SelectItem value="all">{t("All issues")}</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
          <Select value={issueLabelFilter} onValueChange={(value) => setIssueLabelFilter(value as IssueLabelFilter)}>
            <SelectTrigger className="h-8">
              <SelectValue placeholder={t("Label")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="all">{t("Any label")}</SelectItem>
                <SelectItem value="unlabeled">{t("Unlabeled")}</SelectItem>
                {issueLabelOptions.map((label) => (
                  <SelectItem key={label.name} value={issueLabelFilterValue(label.name)}>
                    {label.name} ({label.count})
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Select value={issueAssigneeFilter} onValueChange={(value) => setIssueAssigneeFilter(value as IssueAssigneeFilter)}>
            <SelectTrigger className="h-8">
              <SelectValue placeholder={t("Assignee")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="all">{t("Any assignee")}</SelectItem>
                <SelectItem value="unassigned">{t("Unassigned")}</SelectItem>
                {issueAssigneeOptions.map((assignee) => (
                  <SelectItem key={assignee.id} value={issueAssigneeFilterValue(assignee.id)}>
                    {assignee.label} ({assignee.count})
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Select value={issueSort} onValueChange={(value) => setIssueSort(value as IssueSortMode)}>
            <SelectTrigger className="h-8">
              <SelectValue placeholder={t("Recently updated")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="updated_desc">{t("Recently updated")}</SelectItem>
                <SelectItem value="created_desc">{t("Recently created")}</SelectItem>
                <SelectItem value="number_desc">{t("Newest number")}</SelectItem>
                <SelectItem value="number_asc">{t("Oldest number")}</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div className="mt-2 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
          <span>
            {t("Showing")} {filteredIssues.length} {t("of")} {issueStats.total} {t("issues")}
          </span>
          {hasIssueFilters ? (
            <Button type="button" size="sm" variant="ghost" onClick={resetIssueFilters}>
              {t("Clear filters")}
            </Button>
          ) : null}
        </div>
      </div>

      {!canCreateIssue ? (
        <div className="border-b border-border px-4 py-3">
          <Alert>
            <AlertDescription>{t("Your current project role can inspect issues, but cannot create them.")}</AlertDescription>
          </Alert>
        </div>
      ) : null}

      {isIssueComposerOpen ? (
        <form className="flex flex-col gap-2 border-b border-border px-4 py-3" onSubmit={submitCreateIssue}>
          <Input
            className="h-8"
            placeholder={t("Issue title")}
            value={newIssueTitle}
            onChange={(event) => setNewIssueTitle(event.target.value)}
            required
          />
          <IssueMarkdownEditor
            organizationId={organizationId}
            repoId={repoId}
            t={t}
            value={newIssueDescription}
            placeholder={t("Describe the issue (optional)")}
            onChange={setNewIssueDescription}
            onError={onError}
          />
          <Select value={newIssueAssigneeId} onValueChange={setNewIssueAssigneeId} disabled={users.length === 0}>
            <SelectTrigger className="h-8">
              <SelectValue placeholder={users.length === 0 ? t("No users available") : t("Assign to user (optional)")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {users.map((user) => (
                  <SelectItem key={user.id} value={user.id}>
                    {formatUserLabel(user, user.id)}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <div className="flex justify-end gap-2">
            <Button type="button" size="sm" variant="ghost" onClick={() => setIssueComposerOpen(false)}>
              {t("Cancel")}
            </Button>
            <Button type="submit" size="sm" disabled={!canCreateIssue || isCreatingIssue}>
              {isCreatingIssue ? t("Creating issue...") : t("Create issue")}
            </Button>
          </div>
        </form>
      ) : null}

      {isLoadingIssues ? <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading issues...")}</p> : null}
      {!isLoadingIssues && issues.length === 0 ? (
        <IssueEmptyState
          title={t("No issues yet.")}
          description={canCreateIssue ? t("Create the first issue to start triage for this project.") : t("Issues will appear here after they are created.")}
          actionLabel={canCreateIssue ? t("Create an issue") : undefined}
          onAction={canCreateIssue ? () => setIssueComposerOpen(true) : undefined}
        />
      ) : null}
      {!isLoadingIssues && issues.length > 0 && filteredIssues.length === 0 ? (
        <IssueEmptyState
          title={t("No issues match these filters.")}
          description={t("Try another state, label, assignee, or search term.")}
          actionLabel={hasIssueFilters ? t("Clear filters") : undefined}
          onAction={hasIssueFilters ? resetIssueFilters : undefined}
        />
      ) : null}
      {filteredIssues.length > 0 ? (
        <DataList>
          {filteredIssues.map((issue) => (
            <IssueListItem
              key={issue.id}
              issue={issue}
              userByID={userByID}
              t={t}
              onOpen={() => navigate(buildIssueDetailPath(organizationId, repoId, issue.number))}
            />
          ))}
        </DataList>
      ) : null}
    </Section>
  );
};

const IssueStat = ({ label, value }: { label: string; value: number }) => (
  <div className="flex items-baseline justify-between gap-2 px-4 py-2">
    <span className="text-xs text-muted-foreground">{label}</span>
    <span className="text-sm font-semibold tabular-nums">{value}</span>
  </div>
);

const IssueEmptyState = ({
  title,
  description,
  actionLabel,
  onAction,
}: {
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}) => (
  <div className="px-4 py-10 text-center">
    <p className="text-sm font-medium">{title}</p>
    <p className="mx-auto mt-1 max-w-xl text-sm text-muted-foreground">{description}</p>
    {actionLabel && onAction ? (
      <Button type="button" size="sm" variant="outline" className="mt-3" onClick={onAction}>
        {actionLabel}
      </Button>
    ) : null}
  </div>
);
const IssueListItem = ({
  issue,
  userByID,
  t,
  onOpen,
}: {
  issue: RepositoryIssueView;
  userByID: Map<string, UserView>;
  t: (text: string) => string;
  onOpen: () => void;
}) => {
  const assigneeIDs = issueAssigneeIDs(issue);
  const labels = issue.labels ?? [];
  return (
    <DataRow className="items-stretch p-0">
      <button
        type="button"
        className="flex w-full cursor-pointer items-start gap-4 px-4 py-2.5 text-left"
        onClick={onOpen}
      >
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">
            <span className="mr-1.5 font-mono text-xs text-muted-foreground">#{issue.number}</span>
            {issue.title}
          </p>
          {issue.description ? (
            <p className="mt-0.5 truncate text-xs text-muted-foreground">{issue.description}</p>
          ) : null}
          <p className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
            <span>
              {t("Opened by")} {formatUserLabel(userByID.get(issue.author_user_id), issue.author_user_id)}
            </span>
            <span aria-hidden="true">·</span>
            <span>
              {t("updated")} {formatRelativeTime(issue.updated_at)}
            </span>
            <span aria-hidden="true">·</span>
            <span className="inline-flex items-center gap-1">
              <UserRound className="size-3" aria-hidden="true" />
              {assigneeIDs.length > 0
                ? assigneeIDs.map((userID) => formatUserLabel(userByID.get(userID), userID)).join(", ")
                : t("Unassigned")}
            </span>
          </p>
        </div>
        <div className="flex shrink-0 flex-col items-end gap-1">
          <Badge variant={issue.status === "open" ? "default" : "secondary"} className="gap-1">
            <CircleDot className="size-3" aria-hidden="true" />
            {issue.status === "open" ? t("Open") : t("Closed")}
          </Badge>
          <span className="flex flex-wrap justify-end gap-1">
            {labels.length > 0 ? (
              labels.map((label) => (
                <Badge key={`${label.id}:${label.name}`} variant="secondary" className="gap-1">
                  <Tags className="size-3" aria-hidden="true" />
                  {label.name}
                </Badge>
              ))
            ) : (
              <Badge variant="outline" className="gap-1">
                <Tags className="size-3" aria-hidden="true" />
                {t("No labels")}
              </Badge>
            )}
          </span>
        </div>
      </button>
    </DataRow>
  );
};

const issueAssigneeIDs = (issue: RepositoryIssueView): string[] => {
  const values = issue.assignee_user_ids?.length ? issue.assignee_user_ids : issue.assignee_user_id ? [issue.assignee_user_id] : [];
  return values.map(String).filter(Boolean);
};

const issueLabelFilterValue = (name: string): IssueLabelFilter => `label:${name}`;

const issueAssigneeFilterValue = (userID: string): IssueAssigneeFilter => `user:${userID}`;

const selectedIssueLabelName = (filter: IssueLabelFilter): string => (
  filter.startsWith("label:") ? filter.slice("label:".length) : ""
);

const selectedIssueAssigneeID = (filter: IssueAssigneeFilter): string => (
  filter.startsWith("user:") ? filter.slice("user:".length) : ""
);

const buildIssueLabelFilterOptions = (issues: RepositoryIssueView[]): IssueLabelFilterOption[] => {
  const counts = new Map<string, number>();
  for (const issue of issues) {
    for (const label of issue.labels ?? []) {
      const name = label.name.trim();
      if (name) {
        counts.set(name, (counts.get(name) ?? 0) + 1);
      }
    }
  }
  return Array.from(counts, ([name, count]) => ({ name, count }))
    .sort((left, right) => left.name.localeCompare(right.name));
};

const buildIssueAssigneeFilterOptions = (
  issues: RepositoryIssueView[],
  userByID: Map<string, UserView>,
): IssueAssigneeFilterOption[] => {
  const counts = new Map<string, number>();
  for (const issue of issues) {
    for (const userID of issueAssigneeIDs(issue)) {
      counts.set(userID, (counts.get(userID) ?? 0) + 1);
    }
  }
  return Array.from(counts, ([id, count]) => ({
    id,
    count,
    label: formatUserLabel(userByID.get(id), id),
  })).sort((left, right) => left.label.localeCompare(right.label));
};

const filterAndSortIssuesForTriage = (
  items: RepositoryIssueView[],
  criteria: IssueFilterCriteria,
): RepositoryIssueView[] => {
  const normalizedQuery = normalizeIssueSearch(criteria.query);
  const filtered = items.filter((item) => {
    if (criteria.status !== "all" && item.status !== criteria.status) {
      return false;
    }
    if (!issueMatchesLabelFilter(item, criteria.label)) {
      return false;
    }
    if (!issueMatchesAssigneeFilter(item, criteria.assignee)) {
      return false;
    }
    if (!normalizedQuery) {
      return true;
    }
    return issueMatchesSearch(item, normalizedQuery, criteria.userByID);
  });

  const sorted = [...filtered];
  sorted.sort((left, right) => {
    if (criteria.sort === "updated_desc") {
      return toTimestamp(right.updated_at) - toTimestamp(left.updated_at);
    }
    if (criteria.sort === "created_desc") {
      return toTimestamp(right.created_at) - toTimestamp(left.created_at);
    }
    if (criteria.sort === "number_asc") {
      return left.number - right.number;
    }
    return right.number - left.number;
  });
  return sorted;
};

const issueMatchesLabelFilter = (issue: RepositoryIssueView, filter: IssueLabelFilter): boolean => {
  if (filter === "all") {
    return true;
  }
  const labels = (issue.labels ?? [])
    .map((label) => label.name.trim())
    .filter(Boolean);
  if (filter === "unlabeled") {
    return labels.length === 0;
  }
  return labels.includes(selectedIssueLabelName(filter));
};

const issueMatchesAssigneeFilter = (issue: RepositoryIssueView, filter: IssueAssigneeFilter): boolean => {
  if (filter === "all") {
    return true;
  }
  const assigneeIDs = issueAssigneeIDs(issue);
  if (filter === "unassigned") {
    return assigneeIDs.length === 0;
  }
  return assigneeIDs.includes(selectedIssueAssigneeID(filter));
};

const issueMatchesSearch = (
  issue: RepositoryIssueView,
  normalizedQuery: string,
  userByID: Map<string, UserView>,
): boolean => {
  const assigneeIDs = issueAssigneeIDs(issue);
  const searchableValues = [
    `#${issue.number}`,
    String(issue.number),
    issue.title,
    issue.description ?? "",
    issue.author_user_id,
    formatUserLabel(userByID.get(issue.author_user_id), issue.author_user_id),
    ...(issue.labels ?? []).map((label) => label.name),
    ...assigneeIDs.flatMap((userID) => [userID, formatUserLabel(userByID.get(userID), userID)]),
  ];
  return searchableValues.some((value) => normalizeIssueSearch(value).includes(normalizedQuery));
};

const normalizeIssueSearch = (value: string): string => value.trim().toLowerCase();

/**
 * Orval types `Issue.status` as `string`; the triage UI relies on the narrowed
 * `"open" | "closed"` union of `RepositoryIssueView`.
 */
const toIssueView = (issue: Issue): RepositoryIssueView => ({
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

const toUserView = (user: User): UserView => ({
  id: user.id,
  username: user.username,
  display_name: user.display_name || null,
  email: user.email ?? "",
  status: user.status,
  is_super_admin: user.is_super_admin,
});

