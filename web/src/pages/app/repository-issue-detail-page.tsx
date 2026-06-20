import { useEffect, useMemo, useState } from "react";
import { useCustom, useCustomMutation, useList } from "@refinedev/core";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type {
  RepositoryIssueAssigneeView,
  RepositoryIssueCommentView,
  RepositoryIssueLabelView,
  RepositoryIssueView,
  UserView,
} from "@/pages/types";
import { IssueMarkdownEditor } from "@/pages/app/repository/issue-markdown-editor";
import { renderIssueMarkdown } from "@/pages/app/repository/issue-markdown";
import { extractErrorMessage, formatRelativeTime } from "@/pages/app/repository/issues-utils";
import { buildRepositoryPermissions } from "@/pages/app/repository/repository-permissions";
import {
  isRecord,
  normalizeOptionalString,
  normalizeString,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "@/pages/app/repository/repository-normalizers";

interface MarkdownContentProps {
  content: string;
  organizationId: string;
  repoId: string;
}

const MarkdownContent = ({
  content,
  organizationId,
  repoId,
}: MarkdownContentProps): JSX.Element => {
  const [html, setHtml] = useState("");

  useEffect(() => {
    let active = true;
    renderIssueMarkdown(content || "*No content*", organizationId, repoId)
      .then((next) => {
        if (active) {
          setHtml(next);
        }
      })
      .catch(() => {
        if (active) {
          setHtml("<p>Failed to render markdown.</p>");
        }
      });
    return () => {
      active = false;
    };
  }, [content, organizationId, repoId]);

  return <article className="markdown-body text-sm" dangerouslySetInnerHTML={{ __html: html }} />;
};

export const RepositoryIssueDetailPage = (): JSX.Element => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const params = useParams<{ organizationId: string; projectId?: string; repoId?: string; issueNumber: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.projectId ?? params.repoId ?? "";
  const issueNumber = Number.parseInt(params.issueNumber ?? "", 10);
  const isIssueNumberValid = Number.isFinite(issueNumber) && issueNumber > 0;
  const permissionsQuery = useCustom<RawRecord>({
    url: repoId ? `/projects/${repoId}/permissions` : "/projects/0/permissions",
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  const issueQuery = useCustom<RepositoryIssueView>({
    url: `/projects/${repoId}/issues/${issueNumber}`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId) && isIssueNumberValid,
      refetchOnWindowFocus: false,
    },
  });
  const issue = issueQuery.result.data ?? null;
  const commentsQuery = useCustom<RepositoryIssueCommentView[]>({
    url: issue?.number ? `/projects/${repoId}/issues/${issue.number}/comments` : `/projects/${repoId}/issues/0/comments`,
    method: "get",
    config: { query: { limit: 200 } },
    queryOptions: {
      enabled: Boolean(repoId) && Boolean(issue?.number),
      refetchOnWindowFocus: false,
    },
  });
  const assigneesQuery = useCustom<RawRecord[]>({
    url: issue?.number ? `/projects/${repoId}/issues/${issue.number}/assignees` : `/projects/${repoId}/issues/0/assignees`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId) && Boolean(issue?.number),
      refetchOnWindowFocus: false,
    },
  });
  const labelsQuery = useCustom<RawRecord[]>({
    url: issue?.number ? `/projects/${repoId}/issues/${issue.number}/labels` : `/projects/${repoId}/issues/0/labels`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId) && Boolean(issue?.number),
      refetchOnWindowFocus: false,
    },
  });
  const usersQuery = useList<UserView>({
    resource: "users",
    pagination: { pageSize: 100 },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const comments = commentsQuery.result.data ?? [];
  const assignees = useMemo(
    () => resolveIssueAssignees(assigneesQuery.result.data).map(normalizeIssueAssignee),
    [assigneesQuery.result.data],
  );
  const labels = useMemo(
    () => resolveIssueLabels(labelsQuery.result.data).map(normalizeIssueLabel),
    [labelsQuery.result.data],
  );
  const users = usersQuery.result.data ?? [];
  const userByID = useMemo(
    () => new Map(users.map((user) => [user.id, user])),
    [users],
  );
  const isLoadingIssue = issueQuery.query.isFetching && !issueQuery.query.data;
  const isLoadingComments = commentsQuery.query.isFetching && !commentsQuery.query.data;
  const isLoadingCollaboration = assigneesQuery.query.isFetching || labelsQuery.query.isFetching || usersQuery.query.isFetching;
  const repositoryPermissions = useMemo(
    () => buildRepositoryPermissions(null, false, permissionsQuery.result.data),
    [permissionsQuery.result.data],
  );
  const canWriteIssue = repositoryPermissions.issueWrite;
  const canCommentIssue = repositoryPermissions.issueComment;
  const { mutateAsync: updateIssueStatus, mutation: { isPending: isUpdatingIssue } } = useCustomMutation<RepositoryIssueView>();
  const { mutateAsync: createIssueComment, mutation: { isPending: isCreatingComment } } = useCustomMutation<RepositoryIssueCommentView>();
  const { mutateAsync: setIssueAssignees, mutation: { isPending: isUpdatingAssignees } } = useCustomMutation<RawRecord[]>();
  const { mutateAsync: setIssueLabels, mutation: { isPending: isUpdatingLabels } } = useCustomMutation<RawRecord[]>();
  const [newComment, setNewComment] = useState("");
  const [assigneeDraftUserID, setAssigneeDraftUserID] = useState("");
  const [labelDraftName, setLabelDraftName] = useState("");
  const [labelDraftColor, setLabelDraftColor] = useState("#2563eb");
  const [actionError, setActionError] = useState<string | null>(null);

  const loadIssue = async (): Promise<void> => {
    if (!isIssueNumberValid) {
      setActionError(t("Invalid issue number"));
      return;
    }
    const result = await issueQuery.query.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
      return;
    }
    setActionError(null);
  };

  const loadComments = async (): Promise<void> => {
    if (!issue?.number) {
      return;
    }
    const result = await commentsQuery.query.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
      return;
    }
    setActionError(null);
  };

  const loadCollaboration = async (): Promise<void> => {
    if (!issue?.number) {
      return;
    }
    const [assigneeResult, labelResult] = await Promise.all([assigneesQuery.query.refetch(), labelsQuery.query.refetch()]);
    const error = assigneeResult.error ?? labelResult.error;
    if (error) {
      setActionError(extractErrorMessage(error));
      return;
    }
    setActionError(null);
  };

  const toggleIssueStatus = async () => {
    if (!issue) {
      return;
    }
    const nextStatus = issue.status === "open" ? "closed" : "open";
    setActionError(null);
    try {
      await updateIssueStatus({
        url: `/projects/${repoId}/issues/${issue.number}`,
        method: "patch",
        values: { status: nextStatus },
      });
      await loadIssue();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  const submitComment = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!issue) {
      return;
    }
    const content = newComment.trim();
    if (!content) {
      setActionError(t("Comment content is required"));
      return;
    }
    setActionError(null);
    try {
      await createIssueComment({
        url: `/projects/${repoId}/issues/${issue.number}/comments`,
        method: "post",
        values: { content },
      });
      setNewComment("");
      await loadComments();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  const updateAssignees = async (userIDs: string[]) => {
    if (!issue) {
      return;
    }
    setActionError(null);
    try {
      await setIssueAssignees({
        url: `/projects/${repoId}/issues/${issue.number}/assignees`,
        method: "patch",
        values: {
          user_ids: userIDs.map((item) => Number.parseInt(item, 10)).filter(Number.isFinite),
        },
      });
      setAssigneeDraftUserID("");
      await loadCollaboration();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  const addLabel = async () => {
    if (!issue) {
      return;
    }
    const name = labelDraftName.trim();
    if (!name) {
      setActionError(t("Issue label name is required"));
      return;
    }
    const nextLabels = uniqueByName([
      ...labels.map((label) => ({ name: label.name, color: label.color ?? "" })),
      { name, color: labelDraftColor.trim() },
    ]);
    await updateLabels(nextLabels);
    setLabelDraftName("");
  };

  const removeLabel = async (name: string) => {
    await updateLabels(labels.filter((label) => label.name !== name).map((label) => ({ name: label.name, color: label.color ?? "" })));
  };

  const updateLabels = async (nextLabels: Array<{ name: string; color: string }>) => {
    if (!issue) {
      return;
    }
    setActionError(null);
    try {
      await setIssueLabels({
        url: `/projects/${repoId}/issues/${issue.number}/labels`,
        method: "patch",
        values: {
          labels: nextLabels,
        },
      });
      await loadCollaboration();
    } catch (error) {
      setActionError(extractErrorMessage(error));
    }
  };

  useEffect(() => {
    setActionError(null);
    setNewComment("");
    if (!isIssueNumberValid) {
      setActionError(t("Invalid issue number"));
    }
  }, [repoId, issueNumber, isIssueNumberValid, t]);

  useEffect(() => {
    if (!issueQuery.query.error) {
      return;
    }
    setActionError(extractErrorMessage(issueQuery.query.error));
  }, [issueQuery.query.error]);

  useEffect(() => {
    if (!commentsQuery.query.error) {
      return;
    }
    setActionError(extractErrorMessage(commentsQuery.query.error));
  }, [commentsQuery.query.error]);

  useEffect(() => {
    if (!assigneesQuery.query.error) {
      return;
    }
    setActionError(extractErrorMessage(assigneesQuery.query.error));
  }, [assigneesQuery.query.error]);

  useEffect(() => {
    if (!labelsQuery.query.error) {
      return;
    }
    setActionError(extractErrorMessage(labelsQuery.query.error));
  }, [labelsQuery.query.error]);

  return (
    <div className="space-y-4 page-enter">
      <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
        <Link to="/app/projects" className="underline underline-offset-4">
          {t("My Projects")}
        </Link>
        <span>/</span>
        <Link to={`/app/projects/${organizationId}/${repoId}`} className="underline underline-offset-4">
          {t("Project")}
        </Link>
        <span>/</span>
        <Link to={`/app/projects/${organizationId}/${repoId}?tab=issues`} className="underline underline-offset-4">
          {t("Issues")}
        </Link>
      </div>

      {actionError ? (
        <Alert variant="destructive">
          <AlertDescription>{actionError}</AlertDescription>
        </Alert>
      ) : null}

      {isLoadingIssue ? <p className="text-sm text-muted-foreground">{t("Loading issue...")}</p> : null}

      {!isLoadingIssue && issue ? (
        <Card className="card-enter">
          <CardHeader>
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="space-y-1">
                <CardTitle>
                  #{issue.number} {issue.title}
                </CardTitle>
                <CardDescription>
                  {issue.author_user_id} · {formatRelativeTime(issue.created_at)}
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Badge variant={issue.status === "open" ? "default" : "secondary"}>
                  {issue.status === "open" ? t("Open") : t("Closed")}
                </Badge>
                <Button type="button" size="sm" variant="outline" disabled={!canWriteIssue || isUpdatingIssue} onClick={() => void toggleIssueStatus()}>
                  {issue.status === "open" ? t("Close issue") : t("Reopen issue")}
                </Button>
                <Button type="button" size="sm" variant="outline" onClick={() => navigate(-1)}>
                  {t("Back")}
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="rounded-md border p-3">
              {issue.description ? (
                <MarkdownContent content={issue.description} organizationId={organizationId} repoId={repoId} />
              ) : (
                <p className="text-sm text-muted-foreground">{t("No description provided.")}</p>
              )}
            </div>

            <IssueCollaborationPanel
              assignees={assignees}
              labels={labels}
              users={users}
              userByID={userByID}
              assigneeDraftUserID={assigneeDraftUserID}
              labelDraftName={labelDraftName}
              labelDraftColor={labelDraftColor}
              isLoading={isLoadingCollaboration}
              isUpdatingAssignees={isUpdatingAssignees}
              isUpdatingLabels={isUpdatingLabels}
              canEdit={canWriteIssue}
              t={t}
              onChangeAssigneeDraftUserID={setAssigneeDraftUserID}
              onChangeLabelDraftName={setLabelDraftName}
              onChangeLabelDraftColor={setLabelDraftColor}
              onAddAssignee={() => {
                if (!assigneeDraftUserID) {
                  return;
                }
                void updateAssignees(uniqueStrings([...assignees.map((item) => item.user_id), assigneeDraftUserID]));
              }}
              onRemoveAssignee={(userID) => void updateAssignees(assignees.map((item) => item.user_id).filter((item) => item !== userID))}
              onAddLabel={() => void addLabel()}
              onRemoveLabel={(name) => void removeLabel(name)}
              onReload={() => void loadCollaboration()}
            />

            <div className="space-y-2">
              <p className="text-sm font-medium">{t("Discussion")}</p>
              {isLoadingComments ? <p className="text-xs text-muted-foreground">{t("Loading comments...")}</p> : null}
              {!isLoadingComments && comments.length === 0 ? (
                <p className="text-xs text-muted-foreground">{t("No comments yet.")}</p>
              ) : null}
              {comments.map((comment) => (
                <div key={comment.id} className="rounded-md border p-3">
                  <p className="mb-2 text-xs text-muted-foreground">
                    {comment.author_user_id} · {formatRelativeTime(comment.created_at)}
                  </p>
                  <MarkdownContent content={comment.content} organizationId={organizationId} repoId={repoId} />
                </div>
              ))}
            </div>

            {canCommentIssue ? (
              <form className="space-y-3 rounded-md border p-3" onSubmit={submitComment}>
                <p className="text-sm font-medium">{t("Add a comment")}</p>
                <IssueMarkdownEditor
                  organizationId={organizationId}
                  repoId={repoId}
                  issueId={String(issue.number)}
                  t={t}
                  value={newComment}
                  placeholder={t("Comment with markdown, mention #123, or upload files...")}
                  editorHeight={220}
                  onChange={setNewComment}
                  onError={setActionError}
                />
                <div className="flex justify-end">
                  <Button type="submit" disabled={isCreatingComment}>
                    {isCreatingComment ? t("Commenting...") : t("Comment")}
                  </Button>
                </div>
              </form>
            ) : (
              <Alert>
                <AlertDescription>{t("Your current project role can inspect issues, but cannot comment on them.")}</AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
};

const IssueCollaborationPanel = ({
  assignees,
  labels,
  users,
  userByID,
  assigneeDraftUserID,
  labelDraftName,
  labelDraftColor,
  isLoading,
  isUpdatingAssignees,
  isUpdatingLabels,
  canEdit,
  t,
  onChangeAssigneeDraftUserID,
  onChangeLabelDraftName,
  onChangeLabelDraftColor,
  onAddAssignee,
  onRemoveAssignee,
  onAddLabel,
  onRemoveLabel,
  onReload,
}: {
  assignees: RepositoryIssueAssigneeView[];
  labels: RepositoryIssueLabelView[];
  users: UserView[];
  userByID: Map<string, UserView>;
  assigneeDraftUserID: string;
  labelDraftName: string;
  labelDraftColor: string;
  isLoading: boolean;
  isUpdatingAssignees: boolean;
  isUpdatingLabels: boolean;
  canEdit: boolean;
  t: (text: string) => string;
  onChangeAssigneeDraftUserID: (value: string) => void;
  onChangeLabelDraftName: (value: string) => void;
  onChangeLabelDraftColor: (value: string) => void;
  onAddAssignee: () => void;
  onRemoveAssignee: (userID: string) => void;
  onAddLabel: () => void;
  onRemoveLabel: (name: string) => void;
  onReload: () => void;
}) => {
  const assignedIDs = assignees.map((item) => item.user_id);
  const availableUsers = users.filter((user) => !assignedIDs.includes(user.id));

  return (
    <div className="rounded-md border bg-muted/10 p-3">
      <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="text-sm font-medium">{t("Issue collaboration")}</p>
          <p className="text-xs text-muted-foreground">{t("Manage issue assignees and labels.")}</p>
        </div>
        <Button type="button" size="sm" variant="ghost" disabled={isLoading} onClick={onReload}>
          {isLoading ? t("Loading...") : t("Reload")}
        </Button>
      </div>

      <div className="grid gap-3 lg:grid-cols-2">
        <div className="space-y-3 rounded-md border bg-background/60 p-3">
          <p className="text-sm font-medium">{t("Assignees")}</p>
          <div className="flex flex-wrap gap-2">
            {assignees.length === 0 ? <span className="text-xs text-muted-foreground">{t("No assignees assigned.")}</span> : null}
            {assignees.map((assignee) => (
              <Badge key={assignee.id || assignee.user_id} variant="outline" className="gap-2">
                {formatUserLabel(userByID.get(assignee.user_id), assignee.user_id)}
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="h-4 px-1 text-muted-foreground hover:text-foreground"
                  disabled={!canEdit || isUpdatingAssignees}
                  onClick={() => onRemoveAssignee(assignee.user_id)}
                >
                  x
                </Button>
              </Badge>
            ))}
          </div>
          <div className="grid gap-2 sm:grid-cols-[1fr_auto]">
            <Select
              value={assigneeDraftUserID}
              onValueChange={onChangeAssigneeDraftUserID}
              disabled={!canEdit || isUpdatingAssignees || availableUsers.length === 0}
            >
              <SelectTrigger>
                <SelectValue placeholder={availableUsers.length === 0 ? t("No users available") : t("Select user")} />
              </SelectTrigger>
              <SelectContent>
                {availableUsers.map((user) => (
                  <SelectItem key={user.id} value={user.id}>
                    {formatUserLabel(user, user.id)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button type="button" size="sm" variant="outline" disabled={!canEdit || isUpdatingAssignees || !assigneeDraftUserID} onClick={onAddAssignee}>
              {isUpdatingAssignees ? t("Saving...") : t("Add")}
            </Button>
          </div>
        </div>

        <div className="space-y-3 rounded-md border bg-background/60 p-3">
          <p className="text-sm font-medium">{t("Labels")}</p>
          <div className="flex flex-wrap gap-2">
            {labels.length === 0 ? <span className="text-xs text-muted-foreground">{t("No labels assigned.")}</span> : null}
            {labels.map((label) => (
              <Badge
                key={label.id || label.name}
                variant="outline"
                className="gap-2"
                style={label.color ? { borderColor: label.color, color: label.color } : undefined}
              >
                {label.name}
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="h-4 px-1 text-muted-foreground hover:text-foreground"
                  disabled={!canEdit || isUpdatingLabels}
                  onClick={() => onRemoveLabel(label.name)}
                >
                  x
                </Button>
              </Badge>
            ))}
          </div>
          <div className="grid gap-2 sm:grid-cols-[1fr_116px_auto]">
            <div className="space-y-1">
              <Label className="sr-only" htmlFor="issue-label-name">{t("Label name")}</Label>
              <Input
                id="issue-label-name"
                value={labelDraftName}
                placeholder={t("Label name")}
                disabled={!canEdit}
                onChange={(event) => onChangeLabelDraftName(event.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label className="sr-only" htmlFor="issue-label-color">{t("Label color")}</Label>
              <Input
                id="issue-label-color"
                value={labelDraftColor}
                placeholder="#2563eb"
                disabled={!canEdit}
                onChange={(event) => onChangeLabelDraftColor(event.target.value)}
              />
            </div>
            <Button type="button" size="sm" variant="outline" disabled={!canEdit || isUpdatingLabels || !labelDraftName.trim()} onClick={onAddLabel}>
              {isUpdatingLabels ? t("Saving...") : t("Add")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

const resolveIssueAssignees = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return resolveRecordArray(resolveBody(payload));
};

const resolveIssueLabels = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return resolveRecordArray(resolveBody(payload));
};

const normalizeIssueAssignee = (raw: RawRecord): RepositoryIssueAssigneeView => ({
  id: normalizeString(raw.id ?? raw.ID),
  issue_id: normalizeString(raw.issue_id ?? raw.IssueID),
  user_id: normalizeString(raw.user_id ?? raw.UserID),
  created_at: normalizeString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizeIssueLabel = (raw: RawRecord): RepositoryIssueLabelView => ({
  id: normalizeString(raw.id ?? raw.ID),
  issue_id: normalizeString(raw.issue_id ?? raw.IssueID),
  name: normalizeString(raw.name ?? raw.Name),
  color: normalizeOptionalString(raw.color ?? raw.Color),
  created_at: normalizeString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeString(raw.updated_at ?? raw.UpdatedAt),
});

const formatUserLabel = (user: UserView | undefined, fallbackID: string): string => {
  if (!user) {
    return `#${fallbackID}`;
  }
  const displayName = user.display_name?.trim();
  return displayName ? `${displayName} (@${user.username})` : `@${user.username}`;
};

const uniqueStrings = (values: string[]): string[] => Array.from(new Set(values.filter((value) => value.trim().length > 0)));

const uniqueByName = (labels: Array<{ name: string; color: string }>): Array<{ name: string; color: string }> => {
  const seen = new Set<string>();
  const items: Array<{ name: string; color: string }> = [];
  for (const label of labels) {
    const name = label.name.trim();
    if (!name || seen.has(name)) {
      continue;
    }
    seen.add(name);
    items.push({ name, color: label.color.trim() });
  }
  return items;
};
