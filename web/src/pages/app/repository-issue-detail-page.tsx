import { useEffect, useState } from "react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import type { RepositoryIssueCommentView, RepositoryIssueView } from "@/pages/types";
import { IssueMarkdownEditor } from "@/pages/app/repository/issue-markdown-editor";
import { renderIssueMarkdown } from "@/pages/app/repository/issue-markdown";
import { extractErrorMessage, formatRelativeTime } from "@/pages/app/repository/issues-utils";

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

  const issueQuery = useCustom<RepositoryIssueView>({
    url: `/projects/${repoId}/issues/${issueNumber}`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId) && isIssueNumberValid,
      refetchOnWindowFocus: false,
    },
  });
  const issue = issueQuery.data?.data ?? null;
  const commentsQuery = useCustom<RepositoryIssueCommentView[]>({
    url: issue?.number ? `/projects/${repoId}/issues/${issue.number}/comments` : `/projects/${repoId}/issues/0/comments`,
    method: "get",
    config: { query: { limit: 200 } },
    queryOptions: {
      enabled: Boolean(repoId) && Boolean(issue?.number),
      refetchOnWindowFocus: false,
    },
  });
  const comments = commentsQuery.data?.data ?? [];
  const isLoadingIssue = issueQuery.isFetching && !issueQuery.data;
  const isLoadingComments = commentsQuery.isFetching && !commentsQuery.data;
  const { mutateAsync: updateIssueStatus, isLoading: isUpdatingIssue } = useCustomMutation<RepositoryIssueView>();
  const { mutateAsync: createIssueComment, isLoading: isCreatingComment } = useCustomMutation<RepositoryIssueCommentView>();
  const [newComment, setNewComment] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);

  const loadIssue = async (): Promise<void> => {
    if (!isIssueNumberValid) {
      setActionError(t("Invalid issue number"));
      return;
    }
    const result = await issueQuery.refetch();
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
    const result = await commentsQuery.refetch();
    if (result.error) {
      setActionError(extractErrorMessage(result.error));
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

  useEffect(() => {
    setActionError(null);
    setNewComment("");
    if (!isIssueNumberValid) {
      setActionError(t("Invalid issue number"));
    }
  }, [repoId, issueNumber, isIssueNumberValid, t]);

  useEffect(() => {
    if (!issueQuery.error) {
      return;
    }
    setActionError(extractErrorMessage(issueQuery.error));
  }, [issueQuery.error]);

  useEffect(() => {
    if (!commentsQuery.error) {
      return;
    }
    setActionError(extractErrorMessage(commentsQuery.error));
  }, [commentsQuery.error]);

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
                <Button type="button" size="sm" variant="outline" disabled={isUpdatingIssue} onClick={() => void toggleIssueStatus()}>
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
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
};
