import { useEffect, useMemo, useState } from "react";
import { CheckCircle2, Clock3, GitMerge, GitPullRequest, MessageSquare, RefreshCw, Search, ShieldCheck, ThumbsUp, UserRound, XCircle } from "lucide-react";
import { useCustom, useCustomMutation, useGetIdentity } from "@refinedev/core";
import { Link } from "react-router-dom";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type {
  RepositoryBranchView,
  RepositoryMergeRequestApprovalView,
  RepositoryMergeRequestApprovalRuleCheckView,
  RepositoryMergeRequestApprovalsView,
  RepositoryMergeRequestCheckStatusView,
  RepositoryMergeRequestCommentView,
  RepositoryMergeRequestCommentsView,
  RepositoryMergeRequestDiffView,
  RepositoryMergeRequestParticipantRole,
  RepositoryMergeRequestParticipantsView,
  RepositoryMergeRequestState,
  RepositoryMergeRequestView,
  UserView,
} from "@/pages/types";
import { extractErrorMessage, formatRelativeTime, toTimestamp } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { RepositoryMergeRequestApprovalRulesPanel } from "./repository-merge-request-approval-rules-panel";
import {
  isRecord,
  normalizeBoolean as normalizeBool,
  normalizeNumber,
  normalizeOptionalString,
  normalizeString,
  normalizeStringArray,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "./repository-normalizers";

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
  const mergeRequestsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/merge-requests`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const [selectedIID, setSelectedIID] = useState<number | null>(null);
  const diffQuery = useCustom<RawRecord>({
    url: selectedIID ? `/projects/${repoId}/merge-requests/${selectedIID}/diff` : `/projects/${repoId}/merge-requests/0/diff`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
      refetchOnWindowFocus: false,
    },
  });
  const checksQuery = useCustom<RawRecord>({
    url: selectedIID ? `/projects/${repoId}/merge-requests/${selectedIID}/checks` : `/projects/${repoId}/merge-requests/0/checks`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
      refetchOnWindowFocus: false,
    },
  });
  const participantsQuery = useCustom<RawRecord>({
    url: selectedIID ? `/projects/${repoId}/merge-requests/${selectedIID}/participants` : `/projects/${repoId}/merge-requests/0/participants`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
      refetchOnWindowFocus: false,
    },
  });
  const commentsQuery = useCustom<RawRecord>({
    url: selectedIID ? `/projects/${repoId}/merge-requests/${selectedIID}/comments` : `/projects/${repoId}/merge-requests/0/comments`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
      refetchOnWindowFocus: false,
    },
  });
  const approvalsQuery = useCustom<RawRecord>({
    url: selectedIID ? `/projects/${repoId}/merge-requests/${selectedIID}/approvals` : `/projects/${repoId}/merge-requests/0/approvals`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedIID),
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
  const identityQuery = useGetIdentity<{ id?: string | number }>();
  const { mutateAsync: createMergeRequest, isLoading: isCreating } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateMergeRequest, isLoading: isUpdating } = useCustomMutation<RawRecord>();
  const { mutateAsync: mergeMergeRequest, isLoading: isMerging } = useCustomMutation<RawRecord>();
  const { mutateAsync: setMergeRequestParticipants, isLoading: isUpdatingParticipants } = useCustomMutation<RawRecord>();
  const { mutateAsync: createMergeRequestComment, isLoading: isCreatingComment } = useCustomMutation<RawRecord>();
  const { mutateAsync: approveMergeRequest, isLoading: isUpdatingApproval } = useCustomMutation<RawRecord>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [stateFilter, setStateFilter] = useState<RepositoryMergeRequestState | "all">("opened");
  const [searchQuery, setSearchQuery] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [sourceBranch, setSourceBranch] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "main");
  const [reviewerDraftUserID, setReviewerDraftUserID] = useState("");
  const [assigneeDraftUserID, setAssigneeDraftUserID] = useState("");
  const [newComment, setNewComment] = useState("");

  const mergeRequests = useMemo(
    () => resolveMergeRequestList(mergeRequestsQuery.data?.data).map(normalizeMergeRequest),
    [mergeRequestsQuery.data?.data],
  );
  const selectedMergeRequest = useMemo(
    () => mergeRequests.find((item) => item.iid === selectedIID) ?? mergeRequests[0] ?? null,
    [mergeRequests, selectedIID],
  );
  const filteredMergeRequests = useMemo(
    () => filterMergeRequests(mergeRequests, stateFilter, searchQuery),
    [mergeRequests, stateFilter, searchQuery],
  );
  const stats = useMemo(
    () => ({
      opened: mergeRequests.filter((item) => item.state === "opened").length,
      merged: mergeRequests.filter((item) => item.state === "merged").length,
      closed: mergeRequests.filter((item) => item.state === "closed").length,
    }),
    [mergeRequests],
  );
  const diffView = useMemo(
    () => normalizeDiffView(diffQuery.data?.data),
    [diffQuery.data?.data],
  );
  const checksView = useMemo(
    () => normalizeCheckStatusView(checksQuery.data?.data),
    [checksQuery.data?.data],
  );
  const participantsView = useMemo(
    () => normalizeParticipantsView(participantsQuery.data?.data),
    [participantsQuery.data?.data],
  );
  const commentsView = useMemo(
    () => normalizeCommentsView(commentsQuery.data?.data),
    [commentsQuery.data?.data],
  );
  const approvalsView = useMemo(
    () => normalizeApprovalsView(approvalsQuery.data?.data),
    [approvalsQuery.data?.data],
  );
  const users = useMemo(
    () => resolveUserList(usersQuery.data?.data).map(normalizeUser),
    [usersQuery.data?.data],
  );
  const userByID = useMemo(
    () => new Map(users.map((user) => [user.id, user])),
    [users],
  );
  const participants = participantsView?.participants ?? [];
  const reviewers = useMemo(
    () => participants.filter((item) => item.role === "reviewer"),
    [participants],
  );
  const assignees = useMemo(
    () => participants.filter((item) => item.role === "assignee"),
    [participants],
  );
  const comments = commentsView?.comments ?? [];
  const approvals = approvalsView?.approvals ?? [];
  const currentUserID = identityQuery.data?.id === undefined ? "" : String(identityQuery.data.id);
  const currentUserApproved = Boolean(currentUserID && approvals.some((item) => item.user_id === currentUserID));
  const isMergeBlocked = Boolean(checksView?.required && !checksView.mergeable);
  const isLoadingMergeRequests = mergeRequestsQuery.isFetching && !mergeRequestsQuery.data;
  const canCreateMergeRequest = permissions.mergeRequestCreate;
  const canWriteMergeRequest = permissions.mergeRequestWrite;
  const canCommentMergeRequest = permissions.mergeRequestComment;
  const canMergeMergeRequest = permissions.mergeRequestMerge;

  const loadMergeRequests = async () => {
    const result = await mergeRequestsQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadDiff = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await diffQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadChecks = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await checksQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadParticipants = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await participantsQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadComments = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await commentsQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadApprovals = async () => {
    if (!selectedIID) {
      return;
    }
    const result = await approvalsQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitCreateMergeRequest = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTitle = title.trim();
    const normalizedSource = sourceBranch.trim();
    const normalizedTarget = targetBranch.trim();
    if (!normalizedTitle) return onError(t("Merge request title is required"));
    if (!normalizedSource || !normalizedTarget) return onError(t("Source and target branches are required"));
    if (normalizedSource === normalizedTarget) return onError(t("Source and target branches must differ"));

    onError(null);
    try {
      const response = await createMergeRequest({
        url: `/projects/${repoId}/merge-requests`,
        method: "post",
        values: {
          title: normalizedTitle,
          description: description.trim(),
          source_branch: normalizedSource,
          target_branch: normalizedTarget,
        },
      });
      const created = normalizeMergeRequest(response.data);
      setSelectedIID(created.iid);
      setComposerOpen(false);
      setTitle("");
      setDescription("");
      setSourceBranch("");
      setTargetBranch(defaultBranch || "main");
      await loadMergeRequests();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitUpdateState = async (nextState: RepositoryMergeRequestState) => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await updateMergeRequest({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}`,
        method: "patch",
        values: { state: nextState },
      });
      await loadMergeRequests();
      await Promise.all([loadDiff(), loadChecks(), loadParticipants()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitMerge = async () => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await mergeMergeRequest({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}/merge`,
        method: "post",
        values: {},
      });
      await Promise.all([loadMergeRequests(), onMerged()]);
      await Promise.all([loadDiff(), loadChecks(), loadParticipants()]);
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitSetParticipants = async (role: RepositoryMergeRequestParticipantRole, userIDs: string[]) => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await setMergeRequestParticipants({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}/${role === "reviewer" ? "reviewers" : "assignees"}`,
        method: "patch",
        values: { user_ids: userIDs.map((item) => Number.parseInt(item, 10)).filter(Number.isFinite) },
      });
      if (role === "reviewer") {
        setReviewerDraftUserID("");
      } else {
        setAssigneeDraftUserID("");
      }
      await loadParticipants();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitCreateComment = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedMergeRequest) {
      return;
    }
    const content = newComment.trim();
    if (!content) {
      onError(t("Comment content is required"));
      return;
    }
    onError(null);
    try {
      await createMergeRequestComment({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}/comments`,
        method: "post",
        values: { content },
      });
      setNewComment("");
      await loadComments();
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const submitApproval = async (approved: boolean) => {
    if (!selectedMergeRequest) {
      return;
    }
    onError(null);
    try {
      await approveMergeRequest({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}/${approved ? "approve" : "unapprove"}`,
        method: "post",
        values: {},
      });
      await Promise.all([loadApprovals(), loadChecks()]);
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
    if (selectedIID !== null || mergeRequests.length === 0) {
      return;
    }
    setSelectedIID(mergeRequests[0].iid);
  }, [mergeRequests, selectedIID]);

  useEffect(() => {
    if (!defaultBranch) {
      return;
    }
    setTargetBranch(defaultBranch);
  }, [defaultBranch]);

  useEffect(() => {
    setNewComment("");
  }, [selectedIID]);

  useEffect(() => {
    if (!mergeRequestsQuery.error) {
      return;
    }
    onError(extractErrorMessage(mergeRequestsQuery.error));
  }, [mergeRequestsQuery.error, onError]);

  useEffect(() => {
    if (!diffQuery.error) {
      return;
    }
    onError(extractErrorMessage(diffQuery.error));
  }, [diffQuery.error, onError]);

  useEffect(() => {
    if (!checksQuery.error) {
      return;
    }
    onError(extractErrorMessage(checksQuery.error));
  }, [checksQuery.error, onError]);

  useEffect(() => {
    if (!participantsQuery.error) {
      return;
    }
    onError(extractErrorMessage(participantsQuery.error));
  }, [participantsQuery.error, onError]);

  useEffect(() => {
    if (!commentsQuery.error) {
      return;
    }
    onError(extractErrorMessage(commentsQuery.error));
  }, [commentsQuery.error, onError]);

  useEffect(() => {
    if (!approvalsQuery.error) {
      return;
    }
    onError(extractErrorMessage(approvalsQuery.error));
  }, [approvalsQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Merge Requests")}</CardTitle>
        <CardDescription>{t("Review branch changes and merge them into protected targets.")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <MergeRequestStat label={t("Open")} value={stats.opened} tone="emerald" />
          <MergeRequestStat label={t("Merged")} value={stats.merged} tone="blue" />
          <MergeRequestStat label={t("Closed")} value={stats.closed} tone="slate" />
        </div>

        <div id="merge-request-approval-rules-panel">
          <RepositoryMergeRequestApprovalRulesPanel
            repoId={repoId}
            branches={branches}
            defaultBranch={defaultBranch}
            users={users}
            permissions={permissions}
            t={t}
            onError={onError}
            onRulesChanged={() => void loadChecks()}
          />
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(280px,420px)_1fr]">
          <div className="space-y-3">
            <div className="rounded-md border p-3">
              <div className="grid gap-2 md:grid-cols-[1fr_150px] xl:grid-cols-1">
                <div className="relative">
                  <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    className="pl-8"
                    placeholder={t("Search merge requests")}
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                  />
                </div>
                <Select
                  value={stateFilter}
                  onValueChange={(value) => setStateFilter(value as RepositoryMergeRequestState | "all")}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t("Status")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="opened">{t("Open merge requests")}</SelectItem>
                    <SelectItem value="merged">{t("Merged merge requests")}</SelectItem>
                    <SelectItem value="closed">{t("Closed merge requests")}</SelectItem>
                    <SelectItem value="all">{t("All merge requests")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2">
              <Button
                type="button"
                variant={isComposerOpen ? "secondary" : "outline"}
                disabled={!canCreateMergeRequest}
                onClick={() => setComposerOpen((current) => !current)}
              >
                <GitPullRequest className="size-4" />
                {isComposerOpen ? t("Hide new merge request form") : t("New merge request")}
              </Button>
              <Button type="button" size="sm" variant="ghost" onClick={() => void loadMergeRequests()}>
                <RefreshCw className="size-4" />
                {t("Reload")}
              </Button>
            </div>

            {!canCreateMergeRequest ? (
              <Alert>
                <AlertDescription>{t("Your current project role can inspect merge requests, but cannot create them.")}</AlertDescription>
              </Alert>
            ) : null}

            {isComposerOpen ? (
              <form className="space-y-3 rounded-md border p-3" onSubmit={submitCreateMergeRequest}>
                <Input
                  placeholder={t("Merge request title")}
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                  required
                />
                <Textarea
                  className="min-h-24"
                  placeholder={t("Describe the merge request (optional)")}
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                />
                <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-1">
                  <BranchSelect
                    id="source-branch"
                    label={t("Source branch")}
                    value={sourceBranch}
                    branches={branches}
                    placeholder={t("Select source branch")}
                    onChange={setSourceBranch}
                  />
                  <BranchSelect
                    id="target-branch"
                    label={t("Target branch")}
                    value={targetBranch}
                    branches={branches}
                    placeholder={t("Select target branch")}
                    onChange={setTargetBranch}
                  />
                </div>
                <div className="flex justify-end">
                  <Button type="submit" disabled={!canCreateMergeRequest || isCreating}>
                    {isCreating ? t("Creating merge request...") : t("Create merge request")}
                  </Button>
                </div>
              </form>
            ) : null}

            <div className="space-y-2 rounded-md border p-2">
              {isLoadingMergeRequests ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading merge requests...")}</p>
              ) : null}
              {!isLoadingMergeRequests && filteredMergeRequests.length === 0 ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("No merge requests found.")}</p>
              ) : null}
              {filteredMergeRequests.map((mergeRequest) => (
                <button
                  key={mergeRequest.id}
                  type="button"
                  className={`w-full rounded-md border p-3 text-left transition hover:bg-muted/40 ${
                    selectedMergeRequest?.iid === mergeRequest.iid ? "border-primary/60 bg-primary/5" : ""
                  }`}
                  onClick={() => setSelectedIID(mergeRequest.iid)}
                >
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <p className="font-medium">
                      !{mergeRequest.iid} {mergeRequest.title}
                    </p>
                    <MergeRequestStateBadge state={mergeRequest.state} t={t} />
                  </div>
                  <p className="mt-2 text-xs text-muted-foreground">
                    {mergeRequest.source_branch} → {mergeRequest.target_branch}
                    {mergeRequest.updated_at ? ` · ${formatRelativeTime(mergeRequest.updated_at)}` : ""}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="min-w-0 rounded-md border p-3">
            {!selectedMergeRequest ? (
              <p className="text-sm text-muted-foreground">{t("Select a merge request to inspect the diff.")}</p>
            ) : (
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0 space-y-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="text-lg font-semibold">
                        !{selectedMergeRequest.iid} {selectedMergeRequest.title}
                      </h3>
                      <MergeRequestStateBadge state={selectedMergeRequest.state} t={t} />
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {selectedMergeRequest.source_branch} → {selectedMergeRequest.target_branch}
                    </p>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Button type="button" size="sm" variant="ghost" onClick={() => void loadDiff()}>
                      <RefreshCw className="size-4" />
                      {t("Reload diff")}
                    </Button>
                    {selectedMergeRequest.state === "opened" ? (
                      <>
                        <Button type="button" size="sm" variant="outline" disabled={!canWriteMergeRequest || isUpdating} onClick={() => void submitUpdateState("closed")}>
                          {t("Close merge request")}
                        </Button>
                        <Button type="button" size="sm" disabled={!canMergeMergeRequest || isMerging || isMergeBlocked} onClick={() => void submitMerge()}>
                          <GitMerge className="size-4" />
                          {isMerging ? t("Merging...") : t("Merge")}
                        </Button>
                      </>
                    ) : null}
                    {selectedMergeRequest.state === "closed" ? (
                      <Button type="button" size="sm" variant="outline" disabled={!canWriteMergeRequest || isUpdating} onClick={() => void submitUpdateState("opened")}>
                        {t("Reopen merge request")}
                      </Button>
                    ) : null}
                  </div>
                </div>

                {selectedMergeRequest.description ? (
                  <p className="rounded-md border bg-muted/20 px-3 py-2 text-sm">{selectedMergeRequest.description}</p>
                ) : null}

                <MergeRequestChecksPanel
                  organizationId={organizationId}
                  repoId={repoId}
                  checks={checksView}
                  isLoading={checksQuery.isFetching}
                  t={t}
                  onReload={() => void loadChecks()}
                />

                <MergeRequestParticipantsPanel
                  reviewers={reviewers}
                  assignees={assignees}
                  users={users}
                  userByID={userByID}
                  reviewerDraftUserID={reviewerDraftUserID}
                  assigneeDraftUserID={assigneeDraftUserID}
                  isLoading={participantsQuery.isFetching || usersQuery.isFetching}
                  isUpdating={isUpdatingParticipants}
                  canEdit={canWriteMergeRequest}
                  t={t}
                  onChangeReviewerDraftUserID={setReviewerDraftUserID}
                  onChangeAssigneeDraftUserID={setAssigneeDraftUserID}
                  onAddReviewer={() => {
                    if (!reviewerDraftUserID) return;
                    void submitSetParticipants("reviewer", uniqueStrings([...reviewers.map((item) => item.user_id), reviewerDraftUserID]));
                  }}
                  onRemoveReviewer={(userID) =>
                    void submitSetParticipants("reviewer", reviewers.map((item) => item.user_id).filter((item) => item !== userID))
                  }
                  onAddAssignee={() => {
                    if (!assigneeDraftUserID) return;
                    void submitSetParticipants("assignee", uniqueStrings([...assignees.map((item) => item.user_id), assigneeDraftUserID]));
                  }}
                  onRemoveAssignee={(userID) =>
                    void submitSetParticipants("assignee", assignees.map((item) => item.user_id).filter((item) => item !== userID))
                  }
                  onReload={() => void loadParticipants()}
                />

                <MergeRequestApprovalsPanel
                  approvals={approvals}
                  userByID={userByID}
                  currentUserApproved={currentUserApproved}
                  isLoading={approvalsQuery.isFetching}
                  isUpdating={isUpdatingApproval}
                  canApprove={canWriteMergeRequest && selectedMergeRequest.state === "opened"}
                  t={t}
                  onApprove={() => void submitApproval(true)}
                  onUnapprove={() => void submitApproval(false)}
                  onReload={() => void loadApprovals()}
                />

                <MergeRequestCommentsPanel
                  comments={comments}
                  userByID={userByID}
                  newComment={newComment}
                  isLoading={commentsQuery.isFetching}
                  isCreating={isCreatingComment}
                  canComment={canCommentMergeRequest}
                  t={t}
                  onChangeNewComment={setNewComment}
                  onSubmit={submitCreateComment}
                  onReload={() => void loadComments()}
                />

                <div>
                  <p className="mb-2 text-sm font-medium">{t("Diff")}</p>
                  {diffQuery.isFetching ? (
                    <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading diff...")}</p>
                  ) : (
                    <pre className="max-h-[620px] overflow-auto rounded-md border bg-zinc-950 p-3 text-xs leading-relaxed text-zinc-100">
                      {diffView?.diff?.trim() ? diffView.diff : t("No diff available.")}
                    </pre>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const MergeRequestChecksPanel = ({
  organizationId,
  repoId,
  checks,
  isLoading,
  t,
  onReload,
}: {
  organizationId: string;
  repoId: string;
  checks: RepositoryMergeRequestCheckStatusView | null;
  isLoading: boolean;
  t: (text: string) => string;
  onReload: () => void;
}) => {
  const getRepoTabPath = (tab: string) => (
    `/app/projects/${encodeURIComponent(organizationId)}/${encodeURIComponent(repoId)}?tab=${encodeURIComponent(tab)}`
  );
  const renderBlockerHint = (code: string, message: string) => {
    const commonDetails = [message];
    switch (code) {
      case "approval_rule_unsatisfied":
        return {
          title: t("Approval requirements are not met"),
          details: [
            ...commonDetails,
            t("Have a required reviewer approve this merge request."),
            t("Or adjust approval rules for the target branch."),
          ],
          action: {
            label: t("Review approval rules"),
            href: "#merge-request-approval-rules-panel",
          },
        };
      case "pipeline_repository_missing":
        return {
          title: t("Pipeline service is not configured"),
          details: [
            ...commonDetails,
            t("Enable/attach CI runtime for this repository."),
            t("Create a CI config file in the source branch to trigger pipeline generation."),
          ],
          action: {
            label: t("Open code to add CI config"),
            href: getRepoTabPath("code"),
          },
        };
      case "pipeline_missing":
        return {
          title: t("Pipeline not found"),
          details: [
            ...commonDetails,
            t("Push a new commit or fix CI config to trigger a new pipeline."),
            t("Check the pipeline tab for queued or recent pipeline runs."),
          ],
          action: {
            label: t("Open pipelines"),
            href: getRepoTabPath("pipelines"),
          },
        };
      case "pipeline_not_succeeded":
        return {
          title: t("Pipeline did not succeed"),
          details: [
            ...commonDetails,
            t("Fix pipeline failures and rerun the failed pipeline."),
            t("Open pipeline logs from the Pipelines tab."),
          ],
          action: {
            label: t("Open pipelines"),
            href: getRepoTabPath("pipelines"),
          },
        };
      default:
        return {
          title: t("Policy check blocked"),
          details: commonDetails,
        };
    }
  };
  const renderBlockerAction = (href: string, label: string) => {
    if (href.startsWith("#")) {
      return (
        <a href={href} className="text-xs text-primary underline-offset-4 hover:underline">
          {label}
        </a>
      );
    }
    return (
      <Link to={href} className="text-xs text-primary underline-offset-4 hover:underline">
        {label}
      </Link>
    );
  };
  const meta = checks ? checkStatusMeta(checks) : { label: "Not loaded", className: "border-slate-500/30 bg-slate-500/5", Icon: Clock3 };
  const Icon = meta.Icon;
  return (
    <div className={`rounded-md border p-3 ${meta.className}`}>
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <Icon className="size-4" />
            <p className="font-medium">{t("Merge checks")}</p>
            <Badge variant={checks?.mergeable ? "default" : checks?.required ? "outline" : "secondary"}>{t(meta.label)}</Badge>
          </div>
          <p className="text-xs text-muted-foreground">
            {checks?.blocking_reason ? checks.blocking_reason : t("Branch policy and pipeline state for this merge request.")}
          </p>
        </div>
        <Button type="button" size="sm" variant="ghost" onClick={onReload} disabled={isLoading}>
          <RefreshCw className="size-4" />
          {isLoading ? t("Checking...") : t("Reload checks")}
        </Button>
      </div>
      {checks ? (
        <div className="mt-3 grid gap-2 text-xs text-muted-foreground md:grid-cols-2">
          <CheckLine label={t("Target branch")} value={checks.target_branch || t("N/A")} />
          <CheckLine label={t("Source commit")} value={checks.source_commit_sha ? shortText(checks.source_commit_sha) : t("N/A")} />
          <CheckLine label={t("Protected target")} value={checks.target_branch_protected ? t("Yes") : t("No")} />
          <CheckLine label={t("Pipeline required")} value={checks.pipeline_required ? t("Yes") : t("No")} />
          <CheckLine label={t("Merge request required")} value={checks.require_merge_request ? t("Yes") : t("No")} />
          <CheckLine label={t("Pipeline status")} value={checks.pipeline?.status ? t(checks.pipeline.status) : t(checks.status)} />
        </div>
      ) : null}
      {checks?.blockers?.length ? (
        <div className="mt-3 rounded-md border bg-background/70 px-3 py-2">
          <p className="mb-2 text-xs font-medium text-muted-foreground">{t("Blocking reasons")}</p>
          <div className="space-y-2">
            {checks.blockers.map((blocker) => {
              const blockerHint = renderBlockerHint(blocker.code, blocker.message);
              return (
                <div
                  key={`${blocker.category}:${blocker.code}:${blocker.message}`}
                  className="space-y-1 rounded-md border px-2 py-1.5"
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline">{t(blocker.category || "policy")}</Badge>
                    <span className="text-xs font-medium">{blockerHint.title}</span>
                  </div>
                  <ul className="ml-6 list-disc text-muted-foreground text-xs">
                    {blockerHint.details.map((item, index) => (
                      <li key={`${item}:${index}`}>{item}</li>
                    ))}
                  </ul>
                  {blockerHint.action ? (
                    <div className="mt-1">
                      {renderBlockerAction(blockerHint.action.href, blockerHint.action.label)}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </div>
      ) : null}
      {checks?.approval_rules?.length ? (
        <div className="mt-3 space-y-2">
          <p className="text-xs font-medium text-muted-foreground">
            {t("Approval rules")}: {checks.approval_count}/{checks.required_approvals}
          </p>
          <div className="grid gap-2 md:grid-cols-2">
            {checks.approval_rules.map((rule) => (
              <div key={rule.rule_id || rule.name} className="rounded-md border bg-background/70 px-3 py-2 text-xs">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <span className="font-medium">{rule.name}</span>
                  <Badge variant={rule.satisfied ? "default" : "outline"}>
                    {rule.approval_count}/{rule.approvals_required}
                  </Badge>
                </div>
                <p className="mt-1 text-muted-foreground">
                  {rule.code_owner ? t("CODEOWNERS") : t("Eligible users")} · {rule.target_branch || "*"}
                </p>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
};

const MergeRequestParticipantsPanel = ({
  reviewers,
  assignees,
  users,
  userByID,
  reviewerDraftUserID,
  assigneeDraftUserID,
  isLoading,
  isUpdating,
  canEdit,
  t,
  onChangeReviewerDraftUserID,
  onChangeAssigneeDraftUserID,
  onAddReviewer,
  onRemoveReviewer,
  onAddAssignee,
  onRemoveAssignee,
  onReload,
}: {
  reviewers: RepositoryMergeRequestParticipantsView["participants"];
  assignees: RepositoryMergeRequestParticipantsView["participants"];
  users: UserView[];
  userByID: Map<string, UserView>;
  reviewerDraftUserID: string;
  assigneeDraftUserID: string;
  isLoading: boolean;
  isUpdating: boolean;
  canEdit: boolean;
  t: (text: string) => string;
  onChangeReviewerDraftUserID: (value: string) => void;
  onChangeAssigneeDraftUserID: (value: string) => void;
  onAddReviewer: () => void;
  onRemoveReviewer: (userID: string) => void;
  onAddAssignee: () => void;
  onRemoveAssignee: (userID: string) => void;
  onReload: () => void;
}) => (
  <div className="rounded-md border p-3">
    <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
      <div className="space-y-1">
        <p className="flex items-center gap-2 font-medium">
          <UserRound className="size-4" />
          {t("Reviewers and assignees")}
        </p>
        <p className="text-xs text-muted-foreground">{t("Assign people who should review or own this merge request.")}</p>
      </div>
      <Button type="button" size="sm" variant="ghost" onClick={onReload} disabled={isLoading}>
        <RefreshCw className="size-4" />
        {isLoading ? t("Loading...") : t("Reload")}
      </Button>
    </div>
    <div className="grid gap-3 lg:grid-cols-2">
      <ParticipantRoleSection
        label={t("Reviewers")}
        emptyLabel={t("No reviewers assigned.")}
        selected={reviewers}
        users={users}
        userByID={userByID}
        draftUserID={reviewerDraftUserID}
        isUpdating={isUpdating}
        canEdit={canEdit}
        t={t}
        onChangeDraftUserID={onChangeReviewerDraftUserID}
        onAdd={onAddReviewer}
        onRemove={onRemoveReviewer}
      />
      <ParticipantRoleSection
        label={t("Assignees")}
        emptyLabel={t("No assignees assigned.")}
        selected={assignees}
        users={users}
        userByID={userByID}
        draftUserID={assigneeDraftUserID}
        isUpdating={isUpdating}
        canEdit={canEdit}
        t={t}
        onChangeDraftUserID={onChangeAssigneeDraftUserID}
        onAdd={onAddAssignee}
        onRemove={onRemoveAssignee}
      />
    </div>
  </div>
);

const ParticipantRoleSection = ({
  label,
  emptyLabel,
  selected,
  users,
  userByID,
  draftUserID,
  isUpdating,
  canEdit,
  t,
  onChangeDraftUserID,
  onAdd,
  onRemove,
}: {
  label: string;
  emptyLabel: string;
  selected: RepositoryMergeRequestParticipantsView["participants"];
  users: UserView[];
  userByID: Map<string, UserView>;
  draftUserID: string;
  isUpdating: boolean;
  canEdit: boolean;
  t: (text: string) => string;
  onChangeDraftUserID: (value: string) => void;
  onAdd: () => void;
  onRemove: (userID: string) => void;
}) => {
  const selectedIDs = selected.map((item) => item.user_id);
  const availableUsers = users.filter((user) => !selectedIDs.includes(user.id));
  return (
    <div className="space-y-2 rounded-md border bg-muted/10 p-3">
      <p className="text-sm font-medium">{label}</p>
      <div className="flex flex-wrap gap-2">
        {selected.length === 0 ? <span className="text-xs text-muted-foreground">{emptyLabel}</span> : null}
        {selected.map((item) => (
          <Badge key={`${item.role}-${item.user_id}`} variant="outline" className="gap-2">
            {formatUserLabel(userByID.get(item.user_id), item.user_id)}
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="h-4 px-1 text-muted-foreground hover:text-foreground"
              disabled={!canEdit || isUpdating}
              onClick={() => onRemove(item.user_id)}
            >
              x
            </Button>
          </Badge>
        ))}
      </div>
      <div className="grid gap-2 sm:grid-cols-[1fr_auto]">
        <Select value={draftUserID} onValueChange={onChangeDraftUserID} disabled={!canEdit || isUpdating || availableUsers.length === 0}>
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
        <Button type="button" size="sm" variant="outline" disabled={!canEdit || isUpdating || !draftUserID} onClick={onAdd}>
          {isUpdating ? t("Saving...") : t("Add")}
        </Button>
      </div>
    </div>
  );
};

const MergeRequestApprovalsPanel = ({
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
  <div className="rounded-md border p-3">
    <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
      <div>
        <p className="flex items-center gap-2 font-medium">
          <ThumbsUp className="size-4" />
          {t("Approvals")}
        </p>
        <p className="text-xs text-muted-foreground">{t("Track reviewers who approved this merge request.")}</p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button type="button" size="sm" variant="ghost" disabled={isLoading} onClick={onReload}>
          <RefreshCw className="size-4" />
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
      </div>
    </div>
    <div className="flex flex-wrap gap-2">
      {approvals.length === 0 ? <span className="text-xs text-muted-foreground">{t("No approvals yet.")}</span> : null}
      {approvals.map((approval) => (
        <Badge key={approval.id || approval.user_id} variant="secondary" className="gap-2">
          <CheckCircle2 className="size-3" />
          {formatUserLabel(userByID.get(approval.user_id), approval.user_id)}
        </Badge>
      ))}
    </div>
  </div>
);

const MergeRequestCommentsPanel = ({
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
  <div className="rounded-md border p-3">
    <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
      <div>
        <p className="flex items-center gap-2 font-medium">
          <MessageSquare className="size-4" />
          {t("Discussion")}
        </p>
        <p className="text-xs text-muted-foreground">{t("Review conversation for this merge request.")}</p>
      </div>
      <Button type="button" size="sm" variant="ghost" disabled={isLoading} onClick={onReload}>
        <RefreshCw className="size-4" />
        {isLoading ? t("Loading...") : t("Reload")}
      </Button>
    </div>
    <div className="space-y-2">
      {isLoading ? <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading comments...")}</p> : null}
      {!isLoading && comments.length === 0 ? (
        <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("No comments yet.")}</p>
      ) : null}
      {comments.map((comment) => (
        <div key={comment.id} className="rounded-md border bg-background/60 p-3">
          <p className="mb-1 text-xs text-muted-foreground">
            {formatUserLabel(userByID.get(comment.author_user_id), comment.author_user_id)}
            {comment.created_at ? ` · ${formatRelativeTime(comment.created_at)}` : ""}
          </p>
          <p className="whitespace-pre-wrap text-sm">{comment.body}</p>
        </div>
      ))}
    </div>
    <form className="mt-3 space-y-2" onSubmit={onSubmit}>
      <Textarea
        className="min-h-24"
        value={newComment}
        placeholder={t("Add a merge request comment")}
        disabled={!canComment}
        onChange={(event) => onChangeNewComment(event.target.value)}
      />
      <div className="flex justify-end">
        <Button type="submit" disabled={!canComment || isCreating || !newComment.trim()}>
          {isCreating ? t("Commenting...") : t("Comment")}
        </Button>
      </div>
    </form>
  </div>
);

const BranchSelect = ({
  id,
  label,
  value,
  branches,
  placeholder,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  branches: RepositoryBranchView[];
  placeholder: string;
  onChange: (value: string) => void;
}) => (
  <div className="space-y-1">
    <Label className="text-xs font-medium text-muted-foreground" htmlFor={id}>
      {label}
    </Label>
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {branches.map((branch) => (
          <SelectItem key={branch.name} value={branch.name}>
            {branch.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  </div>
);

const MergeRequestStat = ({ label, value, tone }: { label: string; value: number; tone: "emerald" | "blue" | "slate" }) => {
  const toneClass = {
    emerald: "border-emerald-500/30 bg-emerald-500/5",
    blue: "border-blue-500/30 bg-blue-500/5",
    slate: "border-slate-500/30 bg-slate-500/5",
  }[tone];
  return (
    <div className={`rounded-md border p-3 ${toneClass}`}>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="text-lg font-semibold">{value}</p>
    </div>
  );
};

const MergeRequestStateBadge = ({ state, t }: { state: RepositoryMergeRequestState; t: (text: string) => string }) => {
  const variant = state === "opened" ? "default" : state === "merged" ? "secondary" : "outline";
  const label = state === "opened" ? "Open" : state === "merged" ? "Merged" : "Closed";
  return <Badge variant={variant}>{t(label)}</Badge>;
};

const CheckLine = ({ label, value }: { label: string; value: string }) => (
  <div className="rounded-md border bg-background/60 px-2 py-1">
    <span className="font-medium text-foreground">{label}: </span>
    <span>{value}</span>
  </div>
);

const checkStatusMeta = (checks: RepositoryMergeRequestCheckStatusView) => {
  if (!checks.required) {
    return { label: "No checks required", className: "border-slate-500/30 bg-slate-500/5", Icon: ShieldCheck };
  }
  if (checks.mergeable) {
    return { label: "Ready to merge", className: "border-emerald-500/30 bg-emerald-500/5", Icon: CheckCircle2 };
  }
  if (checks.status === "missing") {
    return { label: "Pipeline missing", className: "border-amber-500/30 bg-amber-500/5", Icon: Clock3 };
  }
  return { label: "Blocked", className: "border-red-500/30 bg-red-500/5", Icon: XCircle };
};

const formatUserLabel = (user: UserView | undefined, fallbackID: string): string => {
  if (!user) {
    return `#${fallbackID}`;
  }
  const displayName = user.display_name?.trim();
  return displayName ? `${displayName} (@${user.username})` : `@${user.username}`;
};

const shortText = (value: string, length = 8): string => (value.length > length ? value.slice(0, length) : value);

const uniqueStrings = (values: string[]): string[] => Array.from(new Set(values.filter((value) => value.trim().length > 0)));

const filterMergeRequests = (
  items: RepositoryMergeRequestView[],
  state: RepositoryMergeRequestState | "all",
  query: string,
): RepositoryMergeRequestView[] => {
  const normalizedQuery = query.trim().toLowerCase();
  return items
    .filter((item) => state === "all" || item.state === state)
    .filter((item) => {
      if (!normalizedQuery) {
        return true;
      }
      return `${item.iid} ${item.title} ${item.description ?? ""} ${item.source_branch} ${item.target_branch}`
        .toLowerCase()
        .includes(normalizedQuery);
    })
    .sort((a, b) => {
      const left = toTimestamp(a.updated_at ?? "");
      const right = toTimestamp(b.updated_at ?? "");
      return (Number.isFinite(right) ? right : 0) - (Number.isFinite(left) ? left : 0);
    });
};

const resolveMergeRequestList = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  if (!isRecord(payload)) {
    return [];
  }
  const nested = payload.body ?? payload.Body;
  if (Array.isArray(nested)) {
    return nested.filter(isRecord);
  }
  return [];
};

const normalizeDiffView = (payload: unknown): RepositoryMergeRequestDiffView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    diff: normalizeString(raw.diff ?? raw.Diff),
  };
};

const normalizeCheckStatusView = (payload: unknown): RepositoryMergeRequestCheckStatusView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    source_branch: normalizeString(raw.source_branch ?? raw.SourceBranch),
    source_commit_sha: normalizeString(raw.source_commit_sha ?? raw.SourceCommitSHA),
    target_branch: normalizeString(raw.target_branch ?? raw.TargetBranch),
    target_branch_protected: normalizeBool(raw.target_branch_protected ?? raw.TargetBranchProtected),
    require_merge_request: normalizeBool(raw.require_merge_request ?? raw.RequireMergeRequest),
    require_pipeline_success: normalizeBool(raw.require_pipeline_success ?? raw.RequirePipelineSuccess),
    pipeline_required: normalizeBool(raw.pipeline_required ?? raw.PipelineRequired),
    required: normalizeBool(raw.required ?? raw.Required),
    mergeable: normalizeBool(raw.mergeable ?? raw.Mergeable),
    status: normalizeString(raw.status ?? raw.Status),
    blocking_reason: normalizeOptionalString(raw.blocking_reason ?? raw.BlockingReason),
    blockers: resolveRecordArray(raw.blockers ?? raw.Blockers).map(normalizeCheckBlocker),
    pipeline: normalizePipeline(raw.pipeline ?? raw.Pipeline),
    required_approvals: normalizeNumber(raw.required_approvals ?? raw.RequiredApprovals),
    approval_count: normalizeNumber(raw.approval_count ?? raw.ApprovalCount),
    approval_rules: resolveRecordArray(raw.approval_rules ?? raw.ApprovalRules).map(normalizeApprovalRuleCheck),
  };
};

const normalizeCheckBlocker = (raw: RawRecord) => ({
  code: normalizeString(raw.code ?? raw.Code),
  category: normalizeString(raw.category ?? raw.Category),
  message: normalizeString(raw.message ?? raw.Message),
});

const normalizeApprovalRuleCheck = (raw: RawRecord): RepositoryMergeRequestApprovalRuleCheckView => ({
  rule_id: normalizeString(raw.rule_id ?? raw.RuleID),
  name: normalizeString(raw.name ?? raw.Name),
  target_branch: normalizeString(raw.target_branch ?? raw.TargetBranch),
  approvals_required: normalizeNumber(raw.approvals_required ?? raw.ApprovalsRequired),
  approval_count: normalizeNumber(raw.approval_count ?? raw.ApprovalCount),
  eligible_user_ids: normalizeStringArray(raw.eligible_user_ids ?? raw.EligibleUserIDs),
  code_owner: normalizeBool(raw.code_owner ?? raw.CodeOwner),
  satisfied: normalizeBool(raw.satisfied ?? raw.Satisfied),
  blocking_reason: normalizeOptionalString(raw.blocking_reason ?? raw.BlockingReason),
});

const normalizeParticipantsView = (payload: unknown): RepositoryMergeRequestParticipantsView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    participants: resolveRecordArray(raw.participants ?? raw.Participants).map(normalizeParticipant),
  };
};

const normalizeCommentsView = (payload: unknown): RepositoryMergeRequestCommentsView | null => {
  const raw = isRecord(payload) ? resolveBody(payload) : payload;
  if (Array.isArray(raw)) {
    return {
      merge_request: normalizeMergeRequest({}),
      comments: raw.filter(isRecord).map(normalizeComment),
    };
  }
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    comments: resolveRecordArray(raw.comments ?? raw.Comments).map(normalizeComment),
  };
};

const normalizeApprovalsView = (payload: unknown): RepositoryMergeRequestApprovalsView | null => {
  const raw = isRecord(payload) ? resolveBody(payload) : payload;
  if (Array.isArray(raw)) {
    return {
      merge_request: normalizeMergeRequest({}),
      approvals: raw.filter(isRecord).map(normalizeApproval),
    };
  }
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    approvals: resolveRecordArray(raw.approvals ?? raw.Approvals).map(normalizeApproval),
  };
};

const normalizeMergeRequest = (rawValue: unknown): RepositoryMergeRequestView => {
  const raw = isRecord(rawValue) ? rawValue : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    iid: normalizeNumber(raw.iid ?? raw.IID),
    author_user_id: normalizeString(raw.author_user_id ?? raw.AuthorUserID),
    title: normalizeString(raw.title ?? raw.Title),
    description: normalizeOptionalString(raw.description ?? raw.Description),
    state: normalizeState(raw.state ?? raw.State),
    source_branch: normalizeString(raw.source_branch ?? raw.SourceBranch),
    target_branch: normalizeString(raw.target_branch ?? raw.TargetBranch),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizeParticipant = (raw: RawRecord) => ({
  id: normalizeString(raw.id ?? raw.ID),
  merge_request_id: normalizeString(raw.merge_request_id ?? raw.MergeRequestID),
  user_id: normalizeString(raw.user_id ?? raw.UserID),
  role: normalizeParticipantRole(raw.role ?? raw.Role),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizeComment = (raw: RawRecord): RepositoryMergeRequestCommentView => ({
  id: normalizeString(raw.id ?? raw.ID),
  merge_request_id: normalizeString(raw.merge_request_id ?? raw.MergeRequestID),
  author_user_id: normalizeString(raw.author_user_id ?? raw.AuthorUserID),
  body: normalizeString(raw.body ?? raw.Body ?? raw.content ?? raw.Content),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizeApproval = (raw: RawRecord): RepositoryMergeRequestApprovalView => ({
  id: normalizeString(raw.id ?? raw.ID),
  merge_request_id: normalizeString(raw.merge_request_id ?? raw.MergeRequestID),
  user_id: normalizeString(raw.user_id ?? raw.UserID),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});

const normalizePipeline = (rawValue: unknown): RepositoryMergeRequestCheckStatusView["pipeline"] => {
  if (!isRecord(rawValue)) {
    return null;
  }
  return {
    id: normalizeString(rawValue.id ?? rawValue.ID),
    project_id: normalizeString(rawValue.project_id ?? rawValue.ProjectID),
    iid: normalizeNumber(rawValue.iid ?? rawValue.IID),
    name: normalizeString(rawValue.name ?? rawValue.Name),
    source: normalizeString(rawValue.source ?? rawValue.Source),
    ref_name: normalizeString(rawValue.ref_name ?? rawValue.RefName),
    commit_sha: normalizeString(rawValue.commit_sha ?? rawValue.CommitSHA),
    status: normalizePipelineStatus(rawValue.status ?? rawValue.Status),
    config_source: normalizeString(rawValue.config_source ?? rawValue.ConfigSource),
    config_content: normalizeOptionalString(rawValue.config_content ?? rawValue.ConfigContent),
    created_at: normalizeOptionalString(rawValue.created_at ?? rawValue.CreatedAt),
    updated_at: normalizeOptionalString(rawValue.updated_at ?? rawValue.UpdatedAt),
    started_at: normalizeOptionalString(rawValue.started_at ?? rawValue.StartedAt),
    finished_at: normalizeOptionalString(rawValue.finished_at ?? rawValue.FinishedAt),
  };
};

const resolveUserList = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  if (!isRecord(payload)) {
    return [];
  }
  return resolveRecordArray(payload.body ?? payload.Body);
};

const normalizeUser = (raw: RawRecord): UserView => ({
  id: normalizeString(raw.id ?? raw.ID),
  username: normalizeString(raw.username ?? raw.Username),
  display_name: normalizeOptionalString(raw.display_name ?? raw.DisplayName),
  email: normalizeString(raw.email ?? raw.Email),
  status: normalizeString(raw.status ?? raw.Status),
  is_super_admin: normalizeBool(raw.is_super_admin ?? raw.IsSuperAdmin),
});

const normalizeState = (value: unknown): RepositoryMergeRequestState => {
  const normalized = normalizeString(value);
  if (normalized === "closed" || normalized === "merged") {
    return normalized;
  }
  return "opened";
};

const normalizeParticipantRole = (value: unknown): RepositoryMergeRequestParticipantRole => {
  const normalized = normalizeString(value);
  return normalized === "assignee" ? "assignee" : "reviewer";
};

const normalizePipelineStatus = (value: unknown) => {
  const normalized = normalizeString(value);
  if (normalized === "canceled") {
    return "cancelled";
  }
  if (normalized === "running" || normalized === "succeeded" || normalized === "failed" || normalized === "cancelled") {
    return normalized;
  }
  return "pending";
};
