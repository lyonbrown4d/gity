import { useEffect, useMemo, useState } from "react";
import { useCustom, useCustomMutation, type HttpError } from "@refinedev/core";
import { useNavigate } from "react-router";
import type { CreateIssue, Issue, IssuePage, User, UserIds, UserPage, Assignee } from "@/lib/generated/model";
import { toRawJsonId } from "@/lib/api-id";
import type { RepositoryIssueView, UserView } from "@/pages/types";
import { buildIssueDetailPath } from "./issue-markdown";
import { extractErrorMessage, type IssueSortMode } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { buildIssueAssigneeFilterOptions, buildIssueLabelFilterOptions, filterAndSortIssuesForTriage, toIssueView, toUserView, type IssueAssigneeFilter, type IssueLabelFilter, type IssueStatusFilter } from "./issues/model";
import { IssueWorkspace } from "./issues/workspace";

interface RepositoryIssuesTabProps {
  organizationId: string;
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryIssuesTab = ({ organizationId, repoId, permissions, t, onError }: RepositoryIssuesTabProps): JSX.Element => {
  const navigate = useNavigate();
  const issuesQuery = useCustom<IssuePage>({
    url: `/projects/${repoId}/issues?state=all&page=1&page_size=100`, method: "get",
    queryOptions: { enabled: Boolean(repoId), refetchOnWindowFocus: false },
  });
  const usersQuery = useCustom<UserPage>({
    url: "/users?page=1&page_size=100", method: "get",
    queryOptions: { enabled: Boolean(repoId), refetchOnWindowFocus: false },
  });
  const createIssueMutation = useCustomMutation<Issue, HttpError, CreateIssue>();
  const setAssigneesMutation = useCustomMutation<Assignee, HttpError, UserIds>();
  const issues = useMemo(() => (issuesQuery.result.data?.items ?? []).map(toIssueView), [issuesQuery.result.data]);
  const users = useMemo(() => (usersQuery.result.data?.items ?? []).map(toUserView), [usersQuery.result.data]);
  const userByID = useMemo(() => new Map(users.map((user) => [user.id, user])), [users]);
  const [status, setStatus] = useState<IssueStatusFilter>("open");
  const [searchQuery, setSearchQuery] = useState("");
  const [labelFilter, setLabelFilter] = useState<IssueLabelFilter>("all");
  const [assigneeFilter, setAssigneeFilter] = useState<IssueAssigneeFilter>("all");
  const [sort, setSort] = useState<IssueSortMode>("updated_desc");
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [assigneeID, setAssigneeID] = useState("");
  const stats = useMemo(() => ({
    total: issues.length,
    open: issues.filter((item) => item.status === "open").length,
    closed: issues.filter((item) => item.status === "closed").length,
    unassigned: issues.filter((item) => !issueAssignees(item).length).length,
    labeled: issues.filter((item) => (item.labels ?? []).length > 0).length,
  }), [issues]);
  const labelOptions = useMemo(() => buildIssueLabelFilterOptions(issues), [issues]);
  const assigneeOptions = useMemo(() => buildIssueAssigneeFilterOptions(issues, userByID), [issues, userByID]);
  const filteredIssues = useMemo(() => filterAndSortIssuesForTriage(issues, {
    status, label: labelFilter, assignee: assigneeFilter, query: searchQuery, sort, userByID,
  }), [issues, status, labelFilter, assigneeFilter, searchQuery, sort, userByID]);
  const hasFilters = status !== "open" || labelFilter !== "all" || assigneeFilter !== "all" || searchQuery.trim().length > 0;
  const isLoading = issuesQuery.query.isFetching && !issuesQuery.result.data;
  const isCreating = createIssueMutation.mutation.isPending || setAssigneesMutation.mutation.isPending;

  const onReload = async () => {
    const result = await issuesQuery.query.refetch();
    onError(result.error ? extractErrorMessage(result.error) : null);
  };
  const resetFilters = () => {
    setStatus("open"); setSearchQuery(""); setLabelFilter("all"); setAssigneeFilter("all"); setSort("updated_desc");
  };
  const submitCreate = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTitle = title.trim();
    if (!normalizedTitle) return onError(t("Issue title is required"));
    onError(null);
    try {
      const created = await createIssueMutation.mutateAsync({
        url: `/projects/${repoId}/issues`, method: "post",
        values: { title: normalizedTitle, description: description.trim() },
      });
      const assignee = assigneeID.trim();
      if (assignee) await setAssigneesMutation.mutateAsync({
        url: `/projects/${repoId}/issues/${created.data.number}/assignees`, method: "patch",
        values: { user_ids: [toRawJsonId(assignee)] },
      });
      setTitle(""); setDescription(""); setAssigneeID(""); setComposerOpen(false);
      await onReload();
      navigate(buildIssueDetailPath(organizationId, repoId, created.data.number));
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  useEffect(() => { if (repoId) onError(null); }, [repoId, onError]);
  useEffect(() => {
    const error = issuesQuery.query.error ?? usersQuery.query.error;
    if (error) onError(extractErrorMessage(error));
  }, [issuesQuery.query.error, usersQuery.query.error, onError]);

  return <IssueWorkspace organizationId={organizationId} repoId={repoId} t={t} onError={onError}
    issues={issues} users={users} userByID={userByID} filteredIssues={filteredIssues} stats={stats}
    labelOptions={labelOptions} assigneeOptions={assigneeOptions} isLoading={isLoading} canCreate={permissions.issueCreate}
    isCreating={isCreating} isComposerOpen={isComposerOpen} setComposerOpen={setComposerOpen}
    title={title} setTitle={setTitle} description={description} setDescription={setDescription} assigneeID={assigneeID} setAssigneeID={setAssigneeID}
    status={status} setStatus={setStatus} searchQuery={searchQuery} setSearchQuery={setSearchQuery}
    labelFilter={labelFilter} setLabelFilter={setLabelFilter} assigneeFilter={assigneeFilter} setAssigneeFilter={setAssigneeFilter}
    sort={sort} setSort={setSort} hasFilters={hasFilters} onResetFilters={resetFilters} onReload={() => void onReload()}
    onCreate={(event) => void submitCreate(event)} onOpenIssue={(issue) => navigate(buildIssueDetailPath(organizationId, repoId, issue.number))} />;
};

const issueAssignees = (issue: RepositoryIssueView): string[] => issue.assignee_user_ids?.length
  ? issue.assignee_user_ids
  : issue.assignee_user_id ? [issue.assignee_user_id] : [];
