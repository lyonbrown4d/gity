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
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
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
import { formatUserLabel, uniqueStrings } from "./repository-user-utils";
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

type MergeRequestStateFilter = RepositoryMergeRequestState | "all";
type MergeRequestBranchFilter = "all" | `branch:${string}`;
type BadgeTone = "default" | "secondary" | "outline";

interface MergeRequestBranchFilterOption {
  name: string;
  count: number;
}

interface MergeRequestFilterCriteria {
  state: MergeRequestStateFilter;
  query: string;
  sourceBranch: MergeRequestBranchFilter;
  targetBranch: MergeRequestBranchFilter;
}

interface MergeRequestListGateSummary {
  gateLabel: string;
  gateVariant: BadgeTone;
  approvalLabel?: string;
  blockerDetail?: string;
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
  const identityQuery = useGetIdentity<{ id?: string | number }>({});
  const { mutateAsync: createMergeRequest, mutation: { isPending: isCreating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateMergeRequest, mutation: { isPending: isUpdating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: mergeMergeRequest, mutation: { isPending: isMerging } } = useCustomMutation<RawRecord>();
  const { mutateAsync: setMergeRequestParticipants, mutation: { isPending: isUpdatingParticipants } } = useCustomMutation<RawRecord>();
  const { mutateAsync: createMergeRequestComment, mutation: { isPending: isCreatingComment } } = useCustomMutation<RawRecord>();
  const { mutateAsync: approveMergeRequest, mutation: { isPending: isUpdatingApproval } } = useCustomMutation<RawRecord>();
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [stateFilter, setStateFilter] = useState<MergeRequestStateFilter>("opened");
  const [searchQuery, setSearchQuery] = useState("");
  const [sourceBranchFilter, setSourceBranchFilter] = useState<MergeRequestBranchFilter>("all");
  const [targetBranchFilter, setTargetBranchFilter] = useState<MergeRequestBranchFilter>("all");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [sourceBranch, setSourceBranch] = useState("");
  const [targetBranch, setTargetBranch] = useState(defaultBranch || "main");
  const [reviewerDraftUserID, setReviewerDraftUserID] = useState("");
  const [assigneeDraftUserID, setAssigneeDraftUserID] = useState("");
  const [newComment, setNewComment] = useState("");

  const mergeRequests = useMemo(
    () => resolveMergeRequestList(mergeRequestsQuery.result.data).map(normalizeMergeRequest),
    [mergeRequestsQuery.result.data],
  );
  const selectedMergeRequest = useMemo(
    () => mergeRequests.find((item) => item.iid === selectedIID) ?? mergeRequests[0] ?? null,
    [mergeRequests, selectedIID],
  );
  const sourceBranchOptions = useMemo(
    () => buildMergeRequestBranchFilterOptions(mergeRequests, "source_branch"),
    [mergeRequests],
  );
  const targetBranchOptions = useMemo(
    () => buildMergeRequestBranchFilterOptions(mergeRequests, "target_branch"),
    [mergeRequests],
  );
  const filteredMergeRequests = useMemo(
    () => filterMergeRequests(mergeRequests, {
      state: stateFilter,
      query: searchQuery,
      sourceBranch: sourceBranchFilter,
      targetBranch: targetBranchFilter,
    }),
    [mergeRequests, stateFilter, searchQuery, sourceBranchFilter, targetBranchFilter],
  );
  const stats = useMemo(
    () => ({
      opened: mergeRequests.filter((item) => item.state === "opened").length,
      merged: mergeRequests.filter((item) => item.state === "merged").length,
      closed: mergeRequests.filter((item) => item.state === "closed").length,
      total: mergeRequests.length,
    }),
    [mergeRequests],
  );
  const diffView = useMemo(
    () => normalizeDiffView(diffQuery.result.data),
    [diffQuery.result.data],
  );
  const checksView = useMemo(
    () => normalizeCheckStatusView(checksQuery.result.data),
    [checksQuery.result.data],
  );
  const selectedChecksView = useMemo(() => {
    if (!checksView || !selectedMergeRequest) {
      return null;
    }
    const checksIID = checksView.merge_request.iid;
    return checksIID === 0 || checksIID === selectedMergeRequest.iid ? checksView : null;
  }, [checksView, selectedMergeRequest]);
  const participantsView = useMemo(
    () => normalizeParticipantsView(participantsQuery.result.data),
    [participantsQuery.result.data],
  );
  const commentsView = useMemo(
    () => normalizeCommentsView(commentsQuery.result.data),
    [commentsQuery.result.data],
  );
  const approvalsView = useMemo(
    () => normalizeApprovalsView(approvalsQuery.result.data),
    [approvalsQuery.result.data],
  );
  const users = useMemo(
    () => resolveUserList(usersQuery.result.data).map(normalizeUser),
    [usersQuery.result.data],
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
  const isMergeBlocked = Boolean(selectedChecksView?.required && !selectedChecksView.mergeable);
  const isLoadingMergeRequests = mergeRequestsQuery.query.isFetching && !mergeRequestsQuery.query.data;
  const canCreateMergeRequest = permissions.mergeRequestCreate;
  const canWriteMergeRequest = permissions.mergeRequestWrite;
  const canCommentMergeRequest = permissions.mergeRequestComment;
  const canMergeMergeRequest = permissions.mergeRequestMerge;
  const hasMergeRequestFilters = stateFilter !== "opened"
    || sourceBranchFilter !== "all"
    || targetBranchFilter !== "all"
    || searchQuery.trim().length > 0;
  const resetMergeRequestFilters = () => {
    setStateFilter("opened");
    setSearchQuery("");
    setSourceBranchFilter("all");
    setTargetBranchFilter("all");
  };
  const loadMergeRequests = async () => {
    const result = await mergeRequestsQuery.query.refetch();
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
    const result = await diffQuery.query.refetch();
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
    const result = await checksQuery.query.refetch();
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
    const result = await participantsQuery.query.refetch();
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
    const result = await commentsQuery.query.refetch();
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
    const result = await approvalsQuery.query.refetch();
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
    if (!mergeRequestsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(mergeRequestsQuery.query.error));
  }, [mergeRequestsQuery.query.error, onError]);

  useEffect(() => {
    if (!diffQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(diffQuery.query.error));
  }, [diffQuery.query.error, onError]);

  useEffect(() => {
    if (!checksQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(checksQuery.query.error));
  }, [checksQuery.query.error, onError]);

  useEffect(() => {
    if (!participantsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(participantsQuery.query.error));
  }, [participantsQuery.query.error, onError]);

  useEffect(() => {
    if (!commentsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(commentsQuery.query.error));
  }, [commentsQuery.query.error, onError]);

  useEffect(() => {
    if (!approvalsQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(approvalsQuery.query.error));
  }, [approvalsQuery.query.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <CardTitle>{t("Merge Requests")}</CardTitle>
        <CardDescription>{t("Review branch changes and merge them into protected targets.")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
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
          <div className="flex flex-col gap-3">
            <div className="rounded-md border p-3">
              <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-1">
                <div className="relative md:col-span-2 xl:col-span-1">
                  <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    className="pl-8"
                    placeholder={t("Search title, branch, or description")}
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                  />
                </div>
                <Select
                  value={stateFilter}
                  onValueChange={(value) => setStateFilter(value as MergeRequestStateFilter)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t("Status")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="opened">{t("Open merge requests")}</SelectItem>
                      <SelectItem value="merged">{t("Merged merge requests")}</SelectItem>
                      <SelectItem value="closed">{t("Closed merge requests")}</SelectItem>
                      <SelectItem value="all">{t("All merge requests")}</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <Select value={sourceBranchFilter} onValueChange={(value) => setSourceBranchFilter(value as MergeRequestBranchFilter)}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("Source branch")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="all">{t("Any source branch")}</SelectItem>
                      {sourceBranchOptions.map((branch) => (
                        <SelectItem key={branch.name} value={mergeRequestBranchFilterValue(branch.name)}>
                          {branch.name} ({branch.count})
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <Select value={targetBranchFilter} onValueChange={(value) => setTargetBranchFilter(value as MergeRequestBranchFilter)}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("Target branch")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="all">{t("Any target branch")}</SelectItem>
                      {targetBranchOptions.map((branch) => (
                        <SelectItem key={branch.name} value={mergeRequestBranchFilterValue(branch.name)}>
                          {branch.name} ({branch.count})
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>
              <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
                <span>
                  {t("Showing")} {filteredMergeRequests.length} {t("of")} {stats.total} {t("merge requests")}
                </span>
                {hasMergeRequestFilters ? (
                  <Button type="button" size="sm" variant="ghost" onClick={resetMergeRequestFilters}>
                    {t("Clear filters")}
                  </Button>
                ) : null}
              </div>
            </div>
            <div className="flex flex-wrap items-center justify-between gap-2">
              <Button
                type="button"
                variant={isComposerOpen ? "secondary" : "outline"}
                disabled={!canCreateMergeRequest}
                onClick={() => setComposerOpen((current) => !current)}
              >
                <GitPullRequest data-icon="inline-start" />
                {isComposerOpen ? t("Hide new merge request form") : t("New merge request")}
              </Button>
              <Button type="button" size="sm" variant="ghost" onClick={() => void loadMergeRequests()}>
                <RefreshCw data-icon="inline-start" />
                {t("Reload")}
              </Button>
            </div>

            {!canCreateMergeRequest ? (
              <Alert>
                <AlertDescription>{t("Your current project role can inspect merge requests, but cannot create them.")}</AlertDescription>
              </Alert>
            ) : null}

            {isComposerOpen ? (
              <form className="flex flex-col gap-3 rounded-md border p-3" onSubmit={submitCreateMergeRequest}>
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

            <div className="flex flex-col gap-2 rounded-md border p-2">
              {isLoadingMergeRequests ? (
                <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading merge requests...")}</p>
              ) : null}
              {!isLoadingMergeRequests && mergeRequests.length === 0 ? (
                <MergeRequestEmptyState
                  title={t("No merge requests yet.")}
                  description={canCreateMergeRequest ? t("Create a merge request to start branch review and merge triage.") : t("Merge requests will appear here after they are created.")}
                  actionLabel={canCreateMergeRequest ? t("Create merge request") : undefined}
                  onAction={canCreateMergeRequest ? () => setComposerOpen(true) : undefined}
                />
              ) : null}
              {!isLoadingMergeRequests && mergeRequests.length > 0 && filteredMergeRequests.length === 0 ? (
                <MergeRequestEmptyState
                  title={t("No merge requests match these filters.")}
                  description={t("Try another state, source branch, target branch, or search term.")}
                  actionLabel={hasMergeRequestFilters ? t("Clear filters") : undefined}
                  onAction={hasMergeRequestFilters ? resetMergeRequestFilters : undefined}
                />
              ) : null}
              {filteredMergeRequests.map((mergeRequest) => {
                const isSelected = selectedMergeRequest?.iid === mergeRequest.iid;
                return (
                  <MergeRequestListItem
                    key={mergeRequest.id || mergeRequest.iid}
                    mergeRequest={mergeRequest}
                    isSelected={isSelected}
                    checks={isSelected ? selectedChecksView : null}
                    approvalsCount={isSelected ? approvals.length : null}
                    isLoadingChecks={isSelected && checksQuery.query.isFetching}
                    t={t}
                    onSelect={() => setSelectedIID(mergeRequest.iid)}
                  />
                );
              })}
            </div>
          </div>

          <div className="min-w-0 rounded-md border p-3">
            {!selectedMergeRequest ? (
              <p className="text-sm text-muted-foreground">{t("Select a merge request to inspect the diff.")}</p>
            ) : (
              <div className="flex flex-col gap-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="flex min-w-0 flex-col gap-1">
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
                      <RefreshCw data-icon="inline-start" />
                      {t("Reload diff")}
                    </Button>
                    {selectedMergeRequest.state === "opened" ? (
                      <>
                        <Button type="button" size="sm" variant="outline" disabled={!canWriteMergeRequest || isUpdating} onClick={() => void submitUpdateState("closed")}>
                          {t("Close merge request")}
                        </Button>
                        <Button type="button" size="sm" disabled={!canMergeMergeRequest || isMerging || isMergeBlocked} onClick={() => void submitMerge()}>
                          <GitMerge data-icon="inline-start" />
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

                <MergeRequestReadinessSummary
                  mergeRequest={selectedMergeRequest}
                  checks={selectedChecksView}
                  approvalsCount={approvals.length}
                  reviewersCount={reviewers.length}
                  assigneesCount={assignees.length}
                  commentsCount={comments.length}
                  currentUserApproved={currentUserApproved}
                  t={t}
                />

                <div className="grid gap-3 2xl:grid-cols-[minmax(0,1fr)_340px]">
                  <div className="flex min-w-0 flex-col gap-3">
                    <MergeRequestChecksPanel
                      organizationId={organizationId}
                      repoId={repoId}
                      checks={selectedChecksView}
                      isLoading={checksQuery.query.isFetching}
                      t={t}
                      onReload={() => void loadChecks()}
                    />

                    <MergeRequestCommentsPanel
                      comments={comments}
                      userByID={userByID}
                      newComment={newComment}
                      isLoading={commentsQuery.query.isFetching}
                      isCreating={isCreatingComment}
                      canComment={canCommentMergeRequest}
                      t={t}
                      onChangeNewComment={setNewComment}
                      onSubmit={submitCreateComment}
                      onReload={() => void loadComments()}
                    />

                    <div className="flex flex-col gap-2">
                      <p className="text-sm font-medium">{t("Diff")}</p>
                      {diffQuery.query.isFetching ? (
                        <p className="rounded-md border px-3 py-2 text-sm text-muted-foreground">{t("Loading diff...")}</p>
                      ) : (
                        <pre className="max-h-[620px] overflow-auto rounded-md border bg-zinc-950 p-3 text-xs leading-relaxed text-zinc-100">
                          {diffView?.diff?.trim() ? diffView.diff : t("No diff available.")}
                        </pre>
                      )}
                    </div>
                  </div>

                  <div className="flex flex-col gap-3">
                    <MergeRequestParticipantsPanel
                      reviewers={reviewers}
                      assignees={assignees}
                      users={users}
                      userByID={userByID}
                      reviewerDraftUserID={reviewerDraftUserID}
                      assigneeDraftUserID={assigneeDraftUserID}
                      isLoading={participantsQuery.query.isFetching || usersQuery.query.isFetching}
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
                      isLoading={approvalsQuery.query.isFetching}
                      isUpdating={isUpdatingApproval}
                      canApprove={canWriteMergeRequest && selectedMergeRequest.state === "opened"}
                      t={t}
                      onApprove={() => void submitApproval(true)}
                      onUnapprove={() => void submitApproval(false)}
                      onReload={() => void loadApprovals()}
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

const MergeRequestEmptyState = ({
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
  <div className="rounded-md border border-dashed bg-muted/20 px-4 py-6 text-center">
    <p className="font-medium">{title}</p>
    <p className="mx-auto mt-1 max-w-xl text-sm text-muted-foreground">{description}</p>
    {actionLabel && onAction ? (
      <Button type="button" size="sm" variant="outline" className="mt-3" onClick={onAction}>
        {actionLabel}
      </Button>
    ) : null}
  </div>
);

const MergeRequestListItem = ({
  mergeRequest,
  isSelected,
  checks,
  approvalsCount,
  isLoadingChecks,
  t,
  onSelect,
}: {
  mergeRequest: RepositoryMergeRequestView;
  isSelected: boolean;
  checks: RepositoryMergeRequestCheckStatusView | null;
  approvalsCount: number | null;
  isLoadingChecks: boolean;
  t: (text: string) => string;
  onSelect: () => void;
}) => {
  const gateSummary = buildMergeRequestListGateSummary(mergeRequest, checks, approvalsCount, isSelected, isLoadingChecks, t);
  return (
    <button
      type="button"
      className={cn(
        "w-full rounded-md border p-3 text-left transition hover:bg-muted/40",
        isSelected ? "border-primary/60 bg-primary/5" : "",
      )}
      onClick={onSelect}
    >
      <div className="flex flex-wrap items-start justify-between gap-2">
        <p className="font-medium">
          !{mergeRequest.iid} {mergeRequest.title}
        </p>
        <MergeRequestStateBadge state={mergeRequest.state} t={t} />
      </div>
      <p className="mt-2 text-xs text-muted-foreground">
        {mergeRequest.updated_at ? `${t("updated")} ${formatRelativeTime(mergeRequest.updated_at)}` : t("No update timestamp")}
      </p>
      <div className="mt-3 flex flex-wrap gap-2">
        <Badge variant="outline">{t("Source")}: {mergeRequest.source_branch || t("N/A")}</Badge>
        <Badge variant="outline">{t("Target")}: {mergeRequest.target_branch || t("N/A")}</Badge>
        <Badge variant={gateSummary.gateVariant}>{gateSummary.gateLabel}</Badge>
        {gateSummary.approvalLabel ? <Badge variant="secondary">{gateSummary.approvalLabel}</Badge> : null}
      </div>
      {gateSummary.blockerDetail ? (
        <p className="mt-2 line-clamp-2 text-xs text-muted-foreground">{gateSummary.blockerDetail}</p>
      ) : null}
    </button>
  );
};

const buildMergeRequestListGateSummary = (
  mergeRequest: RepositoryMergeRequestView,
  checks: RepositoryMergeRequestCheckStatusView | null,
  approvalsCount: number | null,
  isSelected: boolean,
  isLoadingChecks: boolean,
  t: (text: string) => string,
): MergeRequestListGateSummary => {
  if (mergeRequest.state === "merged") {
    return { gateLabel: t("Merged"), gateVariant: "secondary" };
  }
  if (mergeRequest.state === "closed") {
    return { gateLabel: t("Closed"), gateVariant: "outline" };
  }
  if (!isSelected) {
    return { gateLabel: t("Select to inspect gates"), gateVariant: "secondary" };
  }
  if (isLoadingChecks) {
    return { gateLabel: t("Loading gates"), gateVariant: "secondary" };
  }
  if (!checks) {
    return { gateLabel: t("Gate status unavailable"), gateVariant: "outline" };
  }

  const approvalCount = Math.max(checks.approval_count, approvalsCount ?? 0);
  const approvalLabel = checks.required_approvals > 0
    ? `${t("Approvals")} ${approvalCount}/${checks.required_approvals}`
    : t("No approvals required");
  if (!checks.required) {
    return { gateLabel: t("No blockers"), gateVariant: "secondary", approvalLabel };
  }
  if (checks.mergeable) {
    return { gateLabel: t("Ready to merge"), gateVariant: "default", approvalLabel };
  }

  const firstBlockerMessage = checks.blockers
    .map((blocker) => blocker.message.trim())
    .find((message) => message.length > 0);
  const blockerDetail = firstBlockerMessage
    || checks.blocking_reason
    || (checks.pipeline?.status ? `${t("Pipeline")}: ${t(checks.pipeline.status)}` : t("Review merge checks for details."));
  return {
    gateLabel: t("Blocked"),
    gateVariant: "outline",
    approvalLabel,
    blockerDetail,
  };
};
const MergeRequestReadinessSummary = ({
  mergeRequest,
  checks,
  approvalsCount,
  reviewersCount,
  assigneesCount,
  commentsCount,
  currentUserApproved,
  t,
}: {
  mergeRequest: RepositoryMergeRequestView;
  checks: RepositoryMergeRequestCheckStatusView | null;
  approvalsCount: number;
  reviewersCount: number;
  assigneesCount: number;
  commentsCount: number;
  currentUserApproved: boolean;
  t: (text: string) => string;
}) => {
  const isOpen = mergeRequest.state === "opened";
  const statusLabel = !isOpen
    ? mergeRequest.state === "merged" ? "Merged" : "Closed"
    : checks?.mergeable ? "Ready to merge" : checks?.required ? "Blocked" : "Review in progress";
  const requiredApprovals = checks?.required_approvals ?? 0;
  const approvalCount = checks?.approval_count ?? approvalsCount;
  return (
    <div className="rounded-md border bg-card p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="font-medium">{t("Merge readiness")}</p>
          <p className="text-xs text-muted-foreground">{t("GitLab-style merge gate summary for reviewers and maintainers.")}</p>
        </div>
        <Badge variant={isOpen && checks?.mergeable ? "default" : "secondary"}>{t(statusLabel)}</Badge>
      </div>
      <div className="mt-3 grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
        <MergeRequestSummaryItem label={t("Approvals")} value={`${approvalCount}/${requiredApprovals}`} detail={currentUserApproved ? t("You approved") : t("Your approval pending")} />
        <MergeRequestSummaryItem label={t("Reviewers")} value={String(reviewersCount)} detail={assigneesCount > 0 ? `${assigneesCount} ${t("assignee(s)")}` : t("No assignee")} />
        <MergeRequestSummaryItem label={t("Pipeline")} value={checks?.pipeline?.status ? t(checks.pipeline.status) : t(checks?.status || "unknown")} detail={checks?.pipeline_required ? t("Required") : t("Optional")} />
        <MergeRequestSummaryItem label={t("Discussion")} value={String(commentsCount)} detail={t("comment(s)")} />
      </div>
    </div>
  );
};

const MergeRequestSummaryItem = ({ label, value, detail }: { label: string; value: string; detail: string }) => (
  <div className="rounded-md border bg-background/60 px-3 py-2">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-base font-semibold">{value}</p>
    <p className="text-xs text-muted-foreground">{detail}</p>
  </div>
);
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
  const meta = checks ? checkStatusMeta(checks) : { label: "Not loaded", Icon: Clock3 };
  const Icon = meta.Icon;
  return (
    <div className="rounded-md border bg-card p-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
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
          <RefreshCw data-icon="inline-start" />
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
          <div className="flex flex-col gap-2">
            {checks.blockers.map((blocker) => {
              const blockerHint = renderBlockerHint(blocker.code, blocker.message);
              return (
                <div
                  key={`${blocker.category}:${blocker.code}:${blocker.message}`}
                  className="flex flex-col gap-1 rounded-md border px-2 py-1.5"
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
        <div className="mt-3 flex flex-col gap-2">
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
      <div className="flex flex-col gap-1">
        <p className="flex items-center gap-2 font-medium">
          <UserRound className="size-4" />
          {t("Reviewers and assignees")}
        </p>
        <p className="text-xs text-muted-foreground">{t("Assign people who should review or own this merge request.")}</p>
      </div>
      <Button type="button" size="sm" variant="ghost" onClick={onReload} disabled={isLoading}>
        <RefreshCw data-icon="inline-start" />
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
    <div className="flex flex-col gap-2 rounded-md border bg-muted/10 p-3">
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
            <SelectGroup>
              {availableUsers.map((user) => (
                <SelectItem key={user.id} value={user.id}>
                  {formatUserLabel(user, user.id)}
                </SelectItem>
              ))}
            </SelectGroup>
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
        <RefreshCw data-icon="inline-start" />
        {isLoading ? t("Loading...") : t("Reload")}
      </Button>
    </div>
    <div className="flex flex-col gap-2">
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
    <form className="mt-3 flex flex-col gap-2" onSubmit={onSubmit}>
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
  <div className="flex flex-col gap-1">
    <Label className="text-xs font-medium text-muted-foreground" htmlFor={id}>
      {label}
    </Label>
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {branches.map((branch) => (
            <SelectItem key={branch.name} value={branch.name}>
              {branch.name}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  </div>
);

const MergeRequestStat = ({ label, value }: { label: string; value: number; tone: "emerald" | "blue" | "slate" }) => (
  <div className="rounded-md border bg-card p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

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
    return { label: "No checks required", Icon: ShieldCheck };
  }
  if (checks.mergeable) {
    return { label: "Ready to merge", Icon: CheckCircle2 };
  }
  if (checks.status === "missing") {
    return { label: "Pipeline missing", Icon: Clock3 };
  }
  return { label: "Blocked", Icon: XCircle };
};


const shortText = (value: string, length = 8): string => (value.length > length ? value.slice(0, length) : value);


const mergeRequestBranchFilterValue = (branch: string): MergeRequestBranchFilter => `branch:${branch}`;

const selectedMergeRequestBranch = (filter: MergeRequestBranchFilter): string => (
  filter.startsWith("branch:") ? filter.slice("branch:".length) : ""
);

const buildMergeRequestBranchFilterOptions = (
  items: RepositoryMergeRequestView[],
  branchField: "source_branch" | "target_branch",
): MergeRequestBranchFilterOption[] => {
  const counts = new Map<string, number>();
  for (const item of items) {
    const branch = item[branchField].trim();
    if (branch) {
      counts.set(branch, (counts.get(branch) ?? 0) + 1);
    }
  }
  return Array.from(counts, ([name, count]) => ({ name, count }))
    .sort((left, right) => left.name.localeCompare(right.name));
};

const filterMergeRequests = (
  items: RepositoryMergeRequestView[],
  criteria: MergeRequestFilterCriteria,
): RepositoryMergeRequestView[] => {
  const normalizedQuery = criteria.query.trim().toLowerCase();
  return items
    .filter((item) => criteria.state === "all" || item.state === criteria.state)
    .filter((item) => mergeRequestMatchesBranchFilter(item.source_branch, criteria.sourceBranch))
    .filter((item) => mergeRequestMatchesBranchFilter(item.target_branch, criteria.targetBranch))
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

const mergeRequestMatchesBranchFilter = (branch: string, filter: MergeRequestBranchFilter): boolean => {
  if (filter === "all") {
    return true;
  }
  return branch === selectedMergeRequestBranch(filter);
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
