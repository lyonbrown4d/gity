import { useCallback, useState } from "react";
import { GitPullRequest, RefreshCw } from "lucide-react";
import { useLocation, useNavigate, useParams, useSearchParams } from "react-router";
import { Button } from "@/components/ui/button";
import { Toolbar } from "@/components/surface/toolbar";
import type {
  RepositoryBranchView,
} from "@/pages/types";
import { uniqueStrings } from "./repository-user-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { RepositoryMergeRequestApprovalRulesPanel } from "./repository-merge-request-approval-rules-panel";
import { MergeRequestStat } from "./merge-requests/status";
import { MergeRequestListWorkspace } from "./merge-requests/workspace-list";
import { MergeRequestReviewWorkspace } from "./merge-requests/workspace-review";
import { type MergeRequestBranchFilter, type MergeRequestStateFilter } from "./merge-requests/filters";
import { useMergeRequestData } from "./merge-requests/use-data";
import { useMergeRequestActions } from "./merge-requests/use-actions";
import { useMergeRequestLoaders } from "./merge-requests/use-loaders";
import { mergeRequestRoute, parseMergeRequestIID, readMergeRequestFilters, updateMergeRequestFilters } from "./merge-requests/route-state";

interface RepositoryMergeRequestsTabProps {
  organizationId: string;
  repoId: string;
  branches: RepositoryBranchView[];
  defaultBranch: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
  onMerged: () => Promise<void>;
}

export const RepositoryMergeRequestsTab = ({
  organizationId,
  repoId,
  branches,
  defaultBranch,
  permissions,
  t,
  onError,
  onMerged,
}: RepositoryMergeRequestsTabProps): JSX.Element => {
  const navigate = useNavigate();
  const location = useLocation();
  const { mergeRequestIid } = useParams<{ mergeRequestIid?: string }>();
  const selectedIID = parseMergeRequestIID(mergeRequestIid);
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = readMergeRequestFilters(searchParams);
  const stateFilter = filters.state;
  const searchQuery = filters.query;
  const sourceBranchFilter = filters.source;
  const targetBranchFilter = filters.target;
  const updateFilters = useCallback((patch: Parameters<typeof updateMergeRequestFilters>[1]) => {
    setSearchParams((current) => updateMergeRequestFilters(current, patch), { replace: true });
  }, [setSearchParams]);
  const setStateFilter = (value: MergeRequestStateFilter) => updateFilters({ state: value });
  const setSearchQuery = (value: string) => updateFilters({ query: value });
  const setSourceBranchFilter = (value: MergeRequestBranchFilter) => updateFilters({ source: value });
  const setTargetBranchFilter = (value: MergeRequestBranchFilter) => updateFilters({ target: value });
  const setSelectedIID = useCallback((iid: number) => {
    navigate(mergeRequestRoute(organizationId, repoId, iid, location.search));
  }, [navigate, organizationId, repoId, location.search]);
  const backToList = useCallback(() => {
    navigate(mergeRequestRoute(organizationId, repoId, undefined, location.search));
  }, [navigate, organizationId, repoId, location.search]);
  const resetMergeRequestFilters = () => updateFilters({ state: "opened", query: "", source: "all", target: "all" });
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [sourceBranch, setSourceBranch] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "main");
  const [reviewerDraftUserID, setReviewerDraftUserID] = useState("");
  const [assigneeDraftUserID, setAssigneeDraftUserID] = useState("");
  const [newComment, setNewComment] = useState("");

  const data = useMergeRequestData({
    repoId, selectedIID, stateFilter, searchQuery, sourceBranchFilter, targetBranchFilter,
  });
  const {
    mergeRequestsQuery, diffQuery, checksQuery, participantsQuery, commentsQuery,
    discussionsQuery, approvalsQuery, usersQuery, mergeRequests, selectedMergeRequest,
    sourceBranchOptions, targetBranchOptions, filteredMergeRequests, stats, diffView,
    selectedChecksView, inlineThreads, users, userByID, userLabels, reviewers,
    assignees, comments, approvals, currentUserID, currentUserApproved, isLoadingMergeRequests,
  } = data;
  const isMergeBlocked = Boolean(selectedChecksView?.required && !selectedChecksView.mergeable);
  const canCreateMergeRequest = permissions.mergeRequestCreate;
  const canWriteMergeRequest = permissions.mergeRequestWrite;
  const canCommentMergeRequest = permissions.mergeRequestComment;
  const canMergeMergeRequest = permissions.mergeRequestMerge;
  const mergeDisabledReason = !canMergeMergeRequest
    ? t("You do not have permission to merge this request.")
    : checksQuery.query.isFetching
      ? t("Refreshing merge requirements...")
      : !selectedChecksView
        ? checksQuery.query.error
          ? t("Merge requirements could not be loaded. Retry the check request.")
          : t("Load merge requirements before merging.")
    : isMergeBlocked
      ? selectedChecksView?.blocking_reason || t("Required checks must pass before merging.")
      : null;
  const hasMergeRequestFilters = stateFilter !== "opened"
    || sourceBranchFilter !== "all"
    || targetBranchFilter !== "all"
    || searchQuery.trim().length > 0;
  const loaders = useMergeRequestLoaders({
    repoId, selectedIID, defaultBranch, setTargetBranch,
    setNewComment, onError, mergeRequestsQuery, diffQuery, checksQuery,
    participantsQuery, commentsQuery, discussionsQuery, approvalsQuery,
  });
  const {
    loadMergeRequests, loadDiff, loadChecks, loadParticipants, loadComments,
    loadDiscussions, loadApprovals,
  } = loaders;

  const actions = useMergeRequestActions({
    repoId, defaultBranch, selectedMergeRequest, title, description, t, sourceBranch,
    targetBranch, newComment, setSelectedIID, setComposerOpen, setTitle,
    setDescription, setSourceBranch, setTargetBranch, setReviewerDraftUserID,
    setAssigneeDraftUserID, setNewComment, onError, onMerged, loadMergeRequests,
    loadDiff, loadChecks, loadParticipants, loadComments, loadDiscussions, loadApprovals,
  });
  const {
    isCreating, isUpdating, isMerging, isUpdatingParticipants, isCreatingComment,
    isUpdatingApproval, submitCreateMergeRequest, submitUpdateState, submitMerge,
    submitSetParticipants, submitCreateComment, submitInlineComment, submitApproval,
  } = actions;
  const isMergeDisabled = !canMergeMergeRequest || isMerging || !selectedChecksView
    || checksQuery.query.isFetching || Boolean(checksQuery.query.error) || isMergeBlocked;

  return (
    <div className="flex flex-col gap-4 page-enter">
      <section className="overflow-hidden rounded-md border border-border bg-card" aria-labelledby="merge-requests-title">
        <div className="flex flex-col gap-3 border-b border-border px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="min-w-0">
            <h2 id="merge-requests-title" className="text-sm font-semibold leading-tight">{t("Merge Requests")}</h2>
            <p className="mt-1 text-xs text-muted-foreground">{t("Review branch changes and merge them into protected targets.")}</p>
          </div>
          <Toolbar>
            <Button type="button" size="sm" variant={isComposerOpen ? "secondary" : "outline"} disabled={!canCreateMergeRequest} onClick={() => setComposerOpen((current) => !current)}>
              <GitPullRequest data-icon="inline-start" />{isComposerOpen ? t("Hide new merge request form") : t("New merge request")}
            </Button>
            <Button type="button" size="sm" variant="ghost" onClick={() => void loadMergeRequests()}><RefreshCw data-icon="inline-start" />{t("Reload")}</Button>
          </Toolbar>
        </div>
        <div className="grid divide-y divide-border sm:grid-cols-3 sm:divide-x sm:divide-y-0">
          <MergeRequestStat label={t("Open")} value={stats.opened} tone="success" />
          <MergeRequestStat label={t("Merged")} value={stats.merged} tone="primary" />
          <MergeRequestStat label={t("Closed")} value={stats.closed} tone="muted" />
        </div>
      </section>
      <div id="merge-request-approval-rules-panel">
        <RepositoryMergeRequestApprovalRulesPanel repoId={repoId} branches={branches} defaultBranch={defaultBranch} users={users} permissions={permissions} t={t} onError={onError} onRulesChanged={() => void loadChecks()} />
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(280px,420px)_1fr]">
        <MergeRequestListWorkspace
          t={t} branches={branches} isComposerOpen={isComposerOpen} canCreate={canCreateMergeRequest} isCreating={isCreating}
          onToggleComposer={() => setComposerOpen((current) => !current)} onCreate={submitCreateMergeRequest}
          title={title} setTitle={setTitle} description={description} setDescription={setDescription}
          sourceBranch={sourceBranch} setSourceBranch={setSourceBranch} targetBranch={targetBranch} setTargetBranch={setTargetBranch}
          stateFilter={stateFilter} setStateFilter={setStateFilter} sourceBranchFilter={sourceBranchFilter} setSourceBranchFilter={setSourceBranchFilter}
          targetBranchFilter={targetBranchFilter} setTargetBranchFilter={setTargetBranchFilter}
          sourceBranchOptions={sourceBranchOptions} targetBranchOptions={targetBranchOptions}
          searchQuery={searchQuery} setSearchQuery={setSearchQuery} hasFilters={hasMergeRequestFilters} onResetFilters={resetMergeRequestFilters}
          total={stats.total} isLoading={isLoadingMergeRequests} mergeRequests={mergeRequests} filteredMergeRequests={filteredMergeRequests}
          selectedMergeRequest={selectedMergeRequest} selectedChecks={selectedChecksView} approvalsCount={approvals.length}
          isLoadingChecks={checksQuery.query.isFetching} onSelect={setSelectedIID}
        />
        <MergeRequestReviewWorkspace
          organizationId={organizationId} repoId={repoId} permissions={permissions} t={t} mergeRequest={selectedMergeRequest}
          checks={selectedChecksView} approvals={approvals} reviewers={reviewers} assignees={assignees} comments={comments}
          users={users} userByID={userByID} userLabels={userLabels} threads={inlineThreads} diff={diffView?.diff ?? ""}
          diffError={diffQuery.query.error} isLoadingDiff={diffQuery.query.isFetching} isLoadingChecks={checksQuery.query.isFetching}
          isLoadingComments={commentsQuery.query.isFetching} isLoadingParticipants={participantsQuery.query.isFetching || usersQuery.query.isFetching}
          isLoadingApprovals={approvalsQuery.query.isFetching} isUpdating={isUpdating} isMerging={isMerging}
          isCreatingComment={isCreatingComment} isUpdatingParticipants={isUpdatingParticipants} isUpdatingApproval={isUpdatingApproval}
          currentUserApproved={currentUserApproved} reviewerDraftUserID={reviewerDraftUserID} assigneeDraftUserID={assigneeDraftUserID}
          newComment={newComment} mergeDisabledReason={mergeDisabledReason} isMergeDisabled={isMergeDisabled}
          onReloadDiff={() => void loadDiff()} onReloadChecks={() => void loadChecks()} onReloadComments={() => void loadComments()}
          onReloadParticipants={() => void loadParticipants()} onReloadApprovals={() => void loadApprovals()}
          onClose={() => void submitUpdateState("closed")} onReopen={() => void submitUpdateState("opened")} onMerge={() => void submitMerge()}
          onChangeComment={setNewComment} onCreateComment={submitCreateComment} onCreateInlineComment={submitInlineComment}
          onChangeReviewerDraft={setReviewerDraftUserID} onChangeAssigneeDraft={setAssigneeDraftUserID}
          onBackToList={backToList}
          onAddReviewer={() => { if (reviewerDraftUserID) void submitSetParticipants("reviewer", uniqueStrings([...reviewers.map((item) => item.user_id), reviewerDraftUserID])); }}
          onRemoveReviewer={(userID) => void submitSetParticipants("reviewer", reviewers.map((item) => item.user_id).filter((item) => item !== userID))}
          onAddAssignee={() => { if (assigneeDraftUserID) void submitSetParticipants("assignee", uniqueStrings([...assignees.map((item) => item.user_id), assigneeDraftUserID])); }}
          onRemoveAssignee={(userID) => void submitSetParticipants("assignee", assignees.map((item) => item.user_id).filter((item) => item !== userID))}
          onApprove={() => void submitApproval(true)} onUnapprove={() => void submitApproval(false)}
        />
      </div>
    </div>
  );
};
