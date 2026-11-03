import { MessageSquare, RefreshCw, ThumbsUp, CheckCircle2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Toolbar } from "@/components/surface/toolbar";
import { Textarea } from "@/components/ui/textarea";
import type {
  RepositoryMergeRequestApprovalView,
  RepositoryMergeRequestCommentView,
  UserView,
} from "@/pages/types";
import { formatRelativeTime } from "../issues-utils";
import { formatUserLabel } from "../repository-user-utils";

export const MergeRequestApprovalsPanel = ({
  approvals,
  userByID,
  currentUserApproved,
  isLoading,
  isUpdating,
  canApprove,
  t,
  onApprove,
  onUnapprove,
  onReload,
}: {
  approvals: RepositoryMergeRequestApprovalView[];
  userByID: Map<string, UserView>;
  currentUserApproved: boolean;
  isLoading: boolean;
  isUpdating: boolean;
  canApprove: boolean;
  t: (text: string) => string;
  onApprove: () => void;
  onUnapprove: () => void;
  onReload: () => void;
}) => (
  <Section
    title={
      <span className="flex items-center gap-2">
        <ThumbsUp className="size-4" aria-hidden="true" />
        {t("Approvals")}
      </span>
    }
    description={t("Track reviewers who approved this merge request.")}
    actions={
      <Toolbar>
        <Button type="button" size="sm" variant="ghost" disabled={isLoading} onClick={onReload}>
          <RefreshCw data-icon="inline-start" />
          {isLoading ? t("Loading...") : t("Reload")}
        </Button>
        {currentUserApproved ? (
          <Button type="button" size="sm" variant="outline" disabled={isUpdating || !canApprove} onClick={onUnapprove}>
            {isUpdating ? t("Saving...") : t("Unapprove")}
          </Button>
        ) : (
          <Button type="button" size="sm" disabled={isUpdating || !canApprove} onClick={onApprove}>
            {isUpdating ? t("Saving...") : t("Approve")}
          </Button>
        )}
      </Toolbar>
    }
    flush
    className="overflow-hidden"
  >
    <div className="flex flex-wrap gap-1.5 px-4 py-2.5">
      {approvals.length === 0 ? <span className="text-xs text-muted-foreground">{t("No approvals yet.")}</span> : null}
      {approvals.map((approval) => (
        <Badge key={approval.id || approval.user_id} variant="secondary" className="gap-2">
          <CheckCircle2 className="size-3" />
          {formatUserLabel(userByID.get(approval.user_id), approval.user_id)}
        </Badge>
      ))}
    </div>
  </Section>
);

export const MergeRequestCommentsPanel = ({
  comments,
  userByID,
  newComment,
  isLoading,
  isCreating,
  canComment,
  t,
  onChangeNewComment,
  onSubmit,
  onReload,
}: {
  comments: RepositoryMergeRequestCommentView[];
  userByID: Map<string, UserView>;
  newComment: string;
  isLoading: boolean;
  isCreating: boolean;
  canComment: boolean;
  t: (text: string) => string;
  onChangeNewComment: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
  onReload: () => void;
}) => (
  <Section
    title={
      <span className="flex items-center gap-2">
        <MessageSquare className="size-4" aria-hidden="true" />
        {t("Discussion")}
      </span>
    }
    description={t("Review conversation for this merge request.")}
    actions={
      <Button type="button" size="sm" variant="ghost" disabled={isLoading} onClick={onReload}>
        <RefreshCw data-icon="inline-start" />
        {isLoading ? t("Loading...") : t("Reload")}
      </Button>
    }
    flush
    className="overflow-hidden"
  >
    {isLoading ? <p className="px-4 py-2.5 text-sm text-muted-foreground">{t("Loading comments...")}</p> : null}
    {!isLoading && comments.length === 0 ? (
      <p className="px-4 py-2.5 text-sm text-muted-foreground">{t("No comments yet.")}</p>
    ) : null}
    {comments.length > 0 ? (
      <DataList>
        {comments.map((comment) => (
          <DataRow key={comment.id} className="items-start">
            <div className="min-w-0 flex-1">
              <p className="mb-1 text-xs text-muted-foreground">
                {formatUserLabel(userByID.get(comment.author_user_id), comment.author_user_id)}
                {comment.created_at ? ` · ${formatRelativeTime(comment.created_at)}` : ""}
              </p>
              <p className="whitespace-pre-wrap text-sm">{comment.body}</p>
            </div>
          </DataRow>
        ))}
      </DataList>
    ) : null}
    <form className="flex flex-col gap-2 border-t border-border px-4 py-2.5" onSubmit={onSubmit}>
      <Textarea
        className="min-h-24"
        value={newComment}
        placeholder={t("Add a merge request comment")}
        disabled={!canComment}
        onChange={(event) => onChangeNewComment(event.target.value)}
      />
      <div className="flex justify-end">
        <Button type="submit" size="sm" disabled={!canComment || isCreating || !newComment.trim()}>
          {isCreating ? t("Commenting...") : t("Comment")}
        </Button>
      </div>
    </form>
  </Section>
);
