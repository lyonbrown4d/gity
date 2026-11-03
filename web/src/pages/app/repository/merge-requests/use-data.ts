import { useMemo } from "react";
import { useCustom, useGetIdentity } from "@refinedev/core";
import { formatUserLabel } from "../repository-user-utils";
import type { RawRecord } from "../repository-normalizers";
import {
  normalizeApprovalsView, normalizeCheckStatusView, normalizeCommentsView,
  normalizeDiffView, normalizeMergeRequest, normalizeParticipantsView,
  normalizeUser, resolveMergeRequestList, resolveUserList,
} from "./data";
import { buildMergeRequestBranchFilterOptions, filterMergeRequests, type MergeRequestBranchFilter, type MergeRequestStateFilter } from "./filters";
import { resolveInlineReviewThreads } from "./diff-review";

interface UseMergeRequestDataArgs {
  repoId: string;
  selectedIID: number | null;
  stateFilter: MergeRequestStateFilter;
  searchQuery: string;
  sourceBranchFilter: MergeRequestBranchFilter;
  targetBranchFilter: MergeRequestBranchFilter;
}

export const useMergeRequestData = ({
  repoId, selectedIID, stateFilter, searchQuery, sourceBranchFilter, targetBranchFilter,
}: UseMergeRequestDataArgs) => {
  const enabled = Boolean(repoId && selectedIID);
  const mergeRequestsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/merge-requests`, method: "get",
    queryOptions: { enabled: Boolean(repoId), refetchOnWindowFocus: false },
  });
  const diffQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/diff`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const checksQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/checks`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const participantsQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/participants`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const commentsQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/comments`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const discussionsQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/discussions`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const approvalsQuery = useCustom<RawRecord>({
    url: `/projects/${repoId}/merge-requests/${selectedIID ?? 0}/approvals`, method: "get",
    queryOptions: { enabled, refetchOnWindowFocus: false },
  });
  const usersQuery = useCustom<RawRecord[]>({
    url: "/users", method: "get",
    queryOptions: { enabled: Boolean(repoId), refetchOnWindowFocus: false },
  });
  const identityQuery = useGetIdentity<{ id?: string | number }>({});
  const mergeRequests = useMemo(
    () => resolveMergeRequestList(mergeRequestsQuery.result.data).map(normalizeMergeRequest),
    [mergeRequestsQuery.result.data],
  );
  const selectedMergeRequest = useMemo(
    () => mergeRequests.find((item) => item.iid === selectedIID) ?? null,
    [mergeRequests, selectedIID],
  );
  const sourceBranchOptions = useMemo(() => buildMergeRequestBranchFilterOptions(mergeRequests, "source_branch"), [mergeRequests]);
  const targetBranchOptions = useMemo(() => buildMergeRequestBranchFilterOptions(mergeRequests, "target_branch"), [mergeRequests]);
  const filteredMergeRequests = useMemo(
    () => filterMergeRequests(mergeRequests, { state: stateFilter, query: searchQuery, sourceBranch: sourceBranchFilter, targetBranch: targetBranchFilter }),
    [mergeRequests, stateFilter, searchQuery, sourceBranchFilter, targetBranchFilter],
  );
  const stats = useMemo(() => ({
    opened: mergeRequests.filter((item) => item.state === "opened").length,
    merged: mergeRequests.filter((item) => item.state === "merged").length,
    closed: mergeRequests.filter((item) => item.state === "closed").length,
    total: mergeRequests.length,
  }), [mergeRequests]);
  const diffView = useMemo(() => normalizeDiffView(diffQuery.result.data), [diffQuery.result.data]);
  const checksView = useMemo(() => normalizeCheckStatusView(checksQuery.result.data), [checksQuery.result.data]);
  const selectedChecksView = useMemo(() => {
    if (!checksView || !selectedMergeRequest) return null;
    const checksIID = checksView.merge_request.iid;
    return checksIID === 0 || checksIID === selectedMergeRequest.iid ? checksView : null;
  }, [checksView, selectedMergeRequest]);
  const participantsView = useMemo(() => normalizeParticipantsView(participantsQuery.result.data), [participantsQuery.result.data]);
  const commentsView = useMemo(() => normalizeCommentsView(commentsQuery.result.data), [commentsQuery.result.data]);
  const inlineThreads = useMemo(() => resolveInlineReviewThreads(discussionsQuery.result.data), [discussionsQuery.result.data]);
  const approvalsView = useMemo(() => normalizeApprovalsView(approvalsQuery.result.data), [approvalsQuery.result.data]);
  const users = useMemo(() => resolveUserList(usersQuery.result.data).map(normalizeUser), [usersQuery.result.data]);
  const userByID = useMemo(() => new Map(users.map((user) => [user.id, user])), [users]);
  const userLabels = useMemo(() => new Map(users.map((user) => [user.id, formatUserLabel(user)])), [users]);
  const participants = participantsView?.participants ?? [];
  const reviewers = useMemo(() => participants.filter((item) => item.role === "reviewer"), [participants]);
  const assignees = useMemo(() => participants.filter((item) => item.role === "assignee"), [participants]);
  const comments = commentsView?.comments ?? [];
  const approvals = approvalsView?.approvals ?? [];
  const currentUserID = identityQuery.data?.id === undefined ? "" : String(identityQuery.data.id);
  const currentUserApproved = Boolean(currentUserID && approvals.some((item) => item.user_id === currentUserID));
  const isLoadingMergeRequests = mergeRequestsQuery.query.isFetching && !mergeRequestsQuery.query.data;

  return {
    mergeRequestsQuery, diffQuery, checksQuery, participantsQuery, commentsQuery,
    discussionsQuery, approvalsQuery, usersQuery, mergeRequests, selectedMergeRequest,
    sourceBranchOptions, targetBranchOptions, filteredMergeRequests, stats, diffView,
    selectedChecksView, inlineThreads, users, userByID, userLabels, reviewers,
    assignees, comments, approvals, currentUserID, currentUserApproved, isLoadingMergeRequests,
  };
};
