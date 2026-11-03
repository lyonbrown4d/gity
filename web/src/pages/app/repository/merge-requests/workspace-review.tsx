import { useSearchParams } from "react-router";
import { MergeConfirmation } from "./merge-confirmation";
import { SectionTabs, SectionTabPanel } from "@/components/common/section-tabs";
import { ArrowLeft, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/surface/section";
import { Toolbar } from "@/components/surface/toolbar";
import type {
  RepositoryMergeRequestApprovalView,
  RepositoryMergeRequestCheckStatusView,
  RepositoryMergeRequestCommentView,
  RepositoryMergeRequestParticipantsView,
  RepositoryMergeRequestView,
  UserView,
} from "@/pages/types";
import { extractErrorMessage } from "../issues-utils";
import type { RepositoryPermissions } from "../repository-permissions";
import { DiffReview, type InlineReviewThread } from "./diff-review";
import { MergeRequestChecksPanel } from "./checks-panel";
import { MergeRequestParticipantsPanel } from "./participants-panel";
import { MergeRequestApprovalsPanel, MergeRequestCommentsPanel } from "./conversation-panels";
import { MergeRequestReadinessSummary } from "./list-summary";
import { MergeRequestStateBadge } from "./status";

interface MergeRequestReviewWorkspaceProps {
  organizationId: string;
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  mergeRequest: RepositoryMergeRequestView | null;
  checks: RepositoryMergeRequestCheckStatusView | null;
  approvals: RepositoryMergeRequestApprovalView[];
  reviewers: RepositoryMergeRequestParticipantsView["participants"];
  assignees: RepositoryMergeRequestParticipantsView["participants"];
  comments: RepositoryMergeRequestCommentView[];
  users: UserView[];
  userByID: Map<string, UserView>;
  userLabels: ReadonlyMap<string, string>;
  threads: InlineReviewThread[];
  diff: string;
  diffError: unknown;
  isLoadingDiff: boolean;
  isLoadingChecks: boolean;
  isLoadingComments: boolean;
  isLoadingParticipants: boolean;
  isLoadingApprovals: boolean;
  isUpdating: boolean;
  isMerging: boolean;
  isCreatingComment: boolean;
  isUpdatingParticipants: boolean;
  isUpdatingApproval: boolean;
  currentUserApproved: boolean;
  reviewerDraftUserID: string;
  assigneeDraftUserID: string;
  newComment: string;
  mergeDisabledReason: string | null;
  isMergeDisabled: boolean;
  onReloadDiff: () => void;
  onReloadChecks: () => void;
  onReloadComments: () => void;
  onReloadParticipants: () => void;
  onReloadApprovals: () => void;
  onClose: () => void;
  onReopen: () => void;
  onMerge: () => void;
  onChangeComment: (value: string) => void;
  onCreateComment: (event: React.FormEvent<HTMLFormElement>) => void;
  onCreateInlineComment: (body: string, position: { file: string; line: number; column?: number }, discussionID?: string) => Promise<boolean>;
  onChangeReviewerDraft: (value: string) => void;
  onChangeAssigneeDraft: (value: string) => void;
  onAddReviewer: () => void;
  onRemoveReviewer: (userID: string) => void;
  onAddAssignee: () => void;
  onRemoveAssignee: (userID: string) => void;
  onApprove: () => void;
  onUnapprove: () => void;
  onBackToList: () => void;
}

export const MergeRequestReviewWorkspace = (props: MergeRequestReviewWorkspaceProps): JSX.Element => {
  const {
    organizationId, repoId, permissions, t, mergeRequest, checks, approvals,
    reviewers, assignees, comments, users, userByID, userLabels, threads, diff,
    diffError, isLoadingDiff, isLoadingChecks, isLoadingComments, isLoadingParticipants,
    isLoadingApprovals, isUpdating, isMerging, isCreatingComment, isUpdatingParticipants,
    isUpdatingApproval, currentUserApproved, reviewerDraftUserID, assigneeDraftUserID,
    newComment, mergeDisabledReason, isMergeDisabled, onReloadDiff, onReloadChecks,
    onReloadComments, onReloadParticipants, onReloadApprovals, onClose, onReopen,
    onMerge, onChangeComment, onCreateComment, onCreateInlineComment,
    onChangeReviewerDraft, onChangeAssigneeDraft, onAddReviewer, onRemoveReviewer,
    onAddAssignee, onRemoveAssignee, onApprove, onUnapprove, onBackToList,
  } = props;
  const [params, setParams] = useSearchParams();
  const tab = params.get("review") === "discussion" ? "discussion" : "changes";
  const setTab = (value: string) => {
    const next = new URLSearchParams(window.location.search);
    if (value === "discussion") next.set("review", value);
    else next.delete("review");
    setParams(next, { replace: true });
  };
  if (!mergeRequest) {
    return <Section flush className="overflow-hidden"><p className="px-4 py-10 text-sm text-muted-foreground">{t("Select a merge request to inspect the diff.")}</p></Section>;
  }
  const canWrite = permissions.mergeRequestWrite;
  return (
    <div className="flex min-w-0 flex-col gap-3">
      <div className="flex flex-wrap items-start justify-between gap-3 border-b border-border pb-3">
        <div className="flex min-w-0 flex-col gap-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-2xl font-semibold tracking-tight">!{mergeRequest.iid} {mergeRequest.title}</h3>
            <MergeRequestStateBadge state={mergeRequest.state} t={t} />
          </div>
          <p className="text-sm text-muted-foreground">{mergeRequest.source_branch} → {mergeRequest.target_branch}</p>
        </div>
        <Toolbar>
          <Button type="button" size="sm" variant="ghost" onClick={onBackToList}><ArrowLeft data-icon="inline-start" />{t("Back")}</Button>
          <Button type="button" size="sm" variant="ghost" onClick={onReloadDiff}><RefreshCw data-icon="inline-start" />{t("Reload diff")}</Button>
          {mergeRequest.state === "opened" ? <>
            <Button type="button" size="sm" variant="outline" disabled={!canWrite || isUpdating} onClick={onClose}>{t("Close merge request")}</Button>
            <MergeConfirmation source={mergeRequest.source_branch} target={mergeRequest.target_branch} disabled={isMergeDisabled} busy={isMerging} reason={mergeDisabledReason} onMerge={onMerge} t={t} />
          </> : null}
          {mergeRequest.state === "closed" ? <Button type="button" size="sm" variant="outline" disabled={!canWrite || isUpdating} onClick={onReopen}>{t("Reopen merge request")}</Button> : null}
        </Toolbar>
      </div>
      {mergeDisabledReason ? <p id="merge-disabled-reason" className="text-xs text-warning">{mergeDisabledReason}</p> : null}
      {mergeRequest.description ? <p className="bg-muted/30 px-3 py-2 text-sm">{mergeRequest.description}</p> : null}
      <MergeRequestReadinessSummary mergeRequest={mergeRequest} checks={checks} approvalsCount={approvals.length} reviewersCount={reviewers.length} assigneesCount={assignees.length} commentsCount={comments.length} currentUserApproved={currentUserApproved} t={t} />
      <SectionTabs label={t("Merge request sections")} value={tab} onChange={setTab} items={[
        { value: "discussion", label: `${t("Discussion")} (${comments.length})` }, { value: "changes", label: t("File changes") },
      ]}>
      <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="flex min-w-0 flex-col gap-3">
          <SectionTabPanel value="discussion">
          <MergeRequestCommentsPanel comments={comments} userByID={userByID} newComment={newComment} isLoading={isLoadingComments} isCreating={isCreatingComment} canComment={permissions.mergeRequestComment} t={t} onChangeNewComment={onChangeComment} onSubmit={onCreateComment} onReload={onReloadComments} />
          </SectionTabPanel>
          <SectionTabPanel value="changes">
          <Section title={t("Diff")} flush className="overflow-hidden">
            {isLoadingDiff ? <p className="px-3 py-2 text-sm text-muted-foreground">{t("Loading diff...")}</p> : diffError ? <div className="px-3 py-4 text-sm text-destructive">{extractErrorMessage(diffError)}</div> : <DiffReview diff={diff} threads={threads} t={t} canComment={permissions.mergeRequestComment} isSubmitting={isCreatingComment} userLabels={userLabels} onSubmit={onCreateInlineComment} />}
          </Section>
          </SectionTabPanel>
        </div>
        <div className="flex flex-col gap-3 xl:sticky xl:top-20 xl:self-start">
          <MergeRequestChecksPanel organizationId={organizationId} repoId={repoId} checks={checks} isLoading={isLoadingChecks} t={t} onReload={onReloadChecks} />

          <MergeRequestParticipantsPanel reviewers={reviewers} assignees={assignees} users={users} userByID={userByID} reviewerDraftUserID={reviewerDraftUserID} assigneeDraftUserID={assigneeDraftUserID} isLoading={isLoadingParticipants} isUpdating={isUpdatingParticipants} canEdit={canWrite} t={t} onChangeReviewerDraftUserID={onChangeReviewerDraft} onChangeAssigneeDraftUserID={onChangeAssigneeDraft} onAddReviewer={onAddReviewer} onRemoveReviewer={onRemoveReviewer} onAddAssignee={onAddAssignee} onRemoveAssignee={onRemoveAssignee} onReload={onReloadParticipants} />
          <MergeRequestApprovalsPanel approvals={approvals} userByID={userByID} currentUserApproved={currentUserApproved} isLoading={isLoadingApprovals} isUpdating={isUpdatingApproval} canApprove={canWrite && mergeRequest.state === "opened"} t={t} onApprove={onApprove} onUnapprove={onUnapprove} onReload={onReloadApprovals} />
        </div>
      </div>
      </SectionTabs>
    </div>
  );
};
