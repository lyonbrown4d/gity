import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { apiRequest } from "@/lib/api";
import { useI18n } from "@/lib/i18n";
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
  const params = useParams<{ organizationId: string; repoId: string; issueNumber: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.repoId ?? "";
  const issueNumber = Number.parseInt(params.issueNumber ?? "", 10);

  const [issue, setIssue] = useState<RepositoryIssueView | null>(null);
  const [comments, setComments] = useState<RepositoryIssueCommentView[]>([]);
  const [isLoadingIssue, setLoadingIssue] = useState(false);
  const [isLoadingComments, setLoadingComments] = useState(false);
  const [isUpdatingIssue, setUpdatingIssue] = useState(false);
  const [newComment, setNewComment] = useState("");
  const [isCreatingComment, setCreatingComment] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const loadIssue = async () => {
    if (!Number.isFinite(issueNumber) || issueNumber <= 0) {
      setActionError(t("Invalid issue number"));
      return;
    }
    setLoadingIssue(true);
    try {
      const data = await apiRequest<RepositoryIssueView>(`/repos/${repoId}/issues/by-number/${issueNumber}`);
      setIssue(data);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingIssue(false);
    }
  };

  const loadComments = async (issueId: string) => {
    setLoadingComments(true);
    try {
      const query = new URLSearchParams({ limit: "200" });
      const data = await apiRequest<RepositoryIssueCommentView[]>(
        `/repos/${repoId}/issues/${issueId}/comments?${query.toString()}`,
      );
      setComments(data);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setLoadingComments(false);
    }
  };

  const toggleIssueStatus = async () => {
    if (!issue) {
      return;
    }
    const nextStatus = issue.status === "open" ? "closed" : "open";
    setActionError(null);
    setUpdatingIssue(true);
    try {
      const updated = await apiRequest<RepositoryIssueView>(`/repos/${repoId}/issues/${issue.id}`, {
        method: "PATCH",
        body: JSON.stringify({ status: nextStatus }),
      });
      setIssue(updated);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setUpdatingIssue(false);
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
    setCreatingComment(true);
    try {
      await apiRequest<RepositoryIssueCommentView>(`/repos/${repoId}/issues/${issue.id}/comments`, {
        method: "POST",
        body: JSON.stringify({ content }),
      });
      setNewComment("");
      await loadComments(issue.id);
    } catch (error) {
      setActionError(extractErrorMessage(error));
    } finally {
      setCreatingComment(false);
    }
  };

  useEffect(() => {
    setActionError(null);
    setIssue(null);
    setComments([]);
    void loadIssue();
  }, [repoId, issueNumber]);

  useEffect(() => {
    if (!issue?.id) {
      return;
    }
    void loadComments(issue.id);
  }, [issue?.id]);

  return (
    <div className="space-y-4 page-enter">
      <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
        <Link to="/app/repositories" className="underline underline-offset-4">
          {t("My Repositories")}
        </Link>
        <span>/</span>
        <Link to={`/app/repositories/${organizationId}/${repoId}`} className="underline underline-offset-4">
          {t("Repository")}
        </Link>
        <span>/</span>
        <Link to={`/app/repositories/${organizationId}/${repoId}?tab=issues`} className="underline underline-offset-4">
          {t("Issues")}
        </Link>
      </div>

      {actionError ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {actionError}
        </p>
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
                issueId={issue.id}
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
