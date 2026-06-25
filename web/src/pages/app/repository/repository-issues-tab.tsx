import { useEffect, useMemo, useState } from "react";
import { CircleDot, Search, Tags, UserRound } from "lucide-react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { useNavigate } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import { buildIssueDetailPath } from "./issue-markdown";
import { IssueMarkdownEditor } from "./issue-markdown-editor";
import { extractErrorMessage, filterAndSortIssues, formatRelativeTime, type IssueSortMode } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeBoolean, normalizeOptionalString, normalizeString, resolveArrayPayload, type RawRecord } from "./repository-normalizers";
import { formatUserLabel } from "./repository-user-utils";

interface RepositoryIssuesTabProps {
  organizationId: string;
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryIssuesTab = ({
  organizationId,
  repoId,
  permissions,
  t,
  onError,
}: RepositoryIssuesTabProps): JSX.Element => {
  const navigate = useNavigate();
  const issuesQuery = useCustom<RepositoryIssueView[]>({
    url: `/projects/${repoId}/issues`,
    method: "get",
    config: {
      query: {
        status: "all",
        limit: 100,
      },
    },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const usersQuery = useCustom<RawRecord[]>({
    url: "/users",
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: createIssue, mutation: { isPending: isCreatingIssue } } = useCustomMutation<RepositoryIssueView>();
  const issues = useMemo(
    () => resolveArrayPayload<RepositoryIssueView>(issuesQuery.result.data),
    [issuesQuery.result.data],
  );
  const users = useMemo(
    () => resolveUsers(usersQuery.result.data).map(normalizeUser),
    [usersQuery.result.data],
  );
  const userByID = useMemo(
    () => new Map(users.map((user) => [user.id, user])),
    [users],
  );
  const [issueStatusFilter, setIssueStatusFilter] = useState<"open" | "closed" | "all">("open");
  const [issueSearchQuery, setIssueSearchQuery] = useState("");
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
  const filteredIssues = useMemo(
    () => filterAndSortIssues(issues, issueStatusFilter, issueSearchQuery, issueSort),
    [issues, issueStatusFilter, issueSearchQuery, issueSort],
  );
  const isLoadingIssues = issuesQuery.query.isFetching && !issuesQuery.query.data;
  const { mutateAsync: setIssueAssignees } = useCustomMutation<RepositoryIssueView>();
  const canCreateIssue = permissions.issueCreate;

  const loadIssues = async () => {
    const result = await issuesQuery.query.refetch();
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
      const response = await createIssue({
        url: `/projects/${repoId}/issues`,
        method: "post",
        values: {
          title,
          description: newIssueDescription.trim(),
        },
      });
      const assigneeID = newIssueAssigneeId.trim();
      if (assigneeID) {
        await setIssueAssignees({
          url: `/projects/${repoId}/issues/${response.data.number}/assignees`,
          method: "patch",
          values: {
            user_ids: [Number.parseInt(assigneeID, 10)].filter(Number.isFinite),
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
    const error = issuesQuery.query.error ?? usersQuery.query.error;
    if (!error) {
      return;
    }
    onError(extractErrorMessage(error));
  }, [issuesQuery.query.error, usersQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Issues")}</CardTitle>
        <CardDescription>{t("Track bugs, tasks, ownership, labels, and discussions for this project.")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
          <IssueStat label={t("Open")} value={issueStats.open} />
          <IssueStat label={t("Closed")} value={issueStats.closed} />
          <IssueStat label={t("Unassigned")} value={issueStats.unassigned} />
          <IssueStat label={t("Labeled")} value={issueStats.labeled} />
          <IssueStat label={t("Total")} value={issueStats.total} />
        </div>

        <div className="rounded-md border p-3">
          <div className="grid gap-2 md:grid-cols-[1fr_auto_auto]">
            <div className="relative">
              <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                className="pl-8"
                placeholder={t("Search issues")}
                value={issueSearchQuery}
                onChange={(event) => setIssueSearchQuery(event.target.value)}
              />
            </div>
            <Select
              value={issueStatusFilter}
              onValueChange={(value) => setIssueStatusFilter(value as "open" | "closed" | "all")}
            >
              <SelectTrigger>
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
            <Select value={issueSort} onValueChange={(value) => setIssueSort(value as IssueSortMode)}>
              <SelectTrigger>
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
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isIssueComposerOpen ? "secondary" : "outline"}
            disabled={!canCreateIssue}
            onClick={() => setIssueComposerOpen((current) => !current)}
          >
            {isIssueComposerOpen ? t("Hide new issue form") : t("New issue")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadIssues()}>
            {t("Reload")}
          </Button>
        </div>

        {!canCreateIssue ? (
          <Alert>
            <AlertDescription>{t("Your current project role can inspect issues, but cannot create them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isIssueComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border p-3" onSubmit={submitCreateIssue}>
            <Input
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
              <SelectTrigger>
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
            <div className="flex justify-end">
              <Button type="submit" disabled={!canCreateIssue || isCreatingIssue}>
                {isCreatingIssue ? t("Creating issue...") : t("Create issue")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="flex flex-col gap-2 rounded-md border p-2">
          {isLoadingIssues ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading issues...")}</p> : null}
          {!isLoadingIssues && filteredIssues.length === 0 ? (
            <p className="px-2 py-2 text-sm text-muted-foreground">{t("No issues found.")}</p>
          ) : null}
          {filteredIssues.map((issue) => (
            <IssueListItem
              key={issue.id}
              issue={issue}
              userByID={userByID}
              t={t}
              onOpen={() => navigate(buildIssueDetailPath(organizationId, repoId, issue.number))}
            />
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

const IssueStat = ({ label, value }: { label: string; value: number }) => (
  <div className="rounded-md border bg-card p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
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
    <button
      type="button"
      className="w-full rounded-md border p-3 text-left transition hover:bg-muted/40"
      onClick={onOpen}
    >
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="font-medium">
            #{issue.number} {issue.title}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {t("Opened by")} {formatUserLabel(userByID.get(issue.author_user_id), issue.author_user_id)}
            {" · "}
            {t("updated")} {formatRelativeTime(issue.updated_at)}
          </p>
        </div>
        <Badge variant={issue.status === "open" ? "default" : "secondary"} className="gap-1">
          <CircleDot className="size-3" />
          {issue.status === "open" ? t("Open") : t("Closed")}
        </Badge>
      </div>
      {issue.description ? (
        <p className="mt-3 line-clamp-2 text-sm text-muted-foreground">{issue.description}</p>
      ) : null}
      <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
        <span className="inline-flex items-center gap-1">
          <UserRound className="size-3" />
          {assigneeIDs.length > 0
            ? assigneeIDs.map((userID) => formatUserLabel(userByID.get(userID), userID)).join(", ")
            : t("Unassigned")}
        </span>
        <span className="inline-flex items-center gap-1">
          <Tags className="size-3" />
          {labels.length > 0 ? labels.map((label) => label.name).join(", ") : t("No labels")}
        </span>
      </div>
    </button>
  );
};

const issueAssigneeIDs = (issue: RepositoryIssueView): string[] => {
  const values = issue.assignee_user_ids?.length ? issue.assignee_user_ids : issue.assignee_user_id ? [issue.assignee_user_id] : [];
  return values.map(String).filter(Boolean);
};

const resolveUsers = (payload: unknown): RawRecord[] =>
  resolveArrayPayload<unknown>(payload).filter(isRecord);

const normalizeUser = (raw: RawRecord): UserView => ({
  id: normalizeString(raw.id ?? raw.ID),
  username: normalizeString(raw.username ?? raw.Username),
  display_name: normalizeOptionalString(raw.display_name ?? raw.DisplayName),
  email: normalizeString(raw.email ?? raw.Email),
  status: normalizeString(raw.status ?? raw.Status),
  is_super_admin: normalizeBoolean(raw.is_super_admin ?? raw.IsSuperAdmin),
});

