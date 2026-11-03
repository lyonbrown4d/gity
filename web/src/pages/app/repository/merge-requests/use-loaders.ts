import { useEffect } from "react";
import { extractErrorMessage } from "../issues-utils";

interface QueryHandle {
  query: {
    error: unknown;
    refetch: () => Promise<{ error: unknown | null }>;
  };
}

interface UseMergeRequestLoadersArgs {
  repoId: string;
  selectedIID: number | null;
  defaultBranch: string;
  setTargetBranch: (value: string) => void;
  setNewComment: (value: string) => void;
  onError: (message: string | null) => void;
  mergeRequestsQuery: QueryHandle;
  diffQuery: QueryHandle;
  checksQuery: QueryHandle;
  participantsQuery: QueryHandle;
  commentsQuery: QueryHandle;
  discussionsQuery: QueryHandle;
  approvalsQuery: QueryHandle;
}

export const useMergeRequestLoaders = ({
  repoId, selectedIID, defaultBranch, setTargetBranch,
  setNewComment, onError, mergeRequestsQuery, diffQuery, checksQuery,
  participantsQuery, commentsQuery, discussionsQuery, approvalsQuery,
}: UseMergeRequestLoadersArgs) => {
  const load = async (query: QueryHandle, needsSelection = true) => {
    if (needsSelection && !selectedIID) return;
    const result = await query.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadMergeRequests = () => load(mergeRequestsQuery, false);
  const loadDiff = () => load(diffQuery);
  const loadChecks = () => load(checksQuery);
  const loadParticipants = () => load(participantsQuery);
  const loadComments = () => load(commentsQuery);
  const loadDiscussions = () => load(discussionsQuery);
  const loadApprovals = () => load(approvalsQuery);

  useEffect(() => {
    if (repoId) onError(null);
  }, [repoId, onError]);
  useEffect(() => {
    if (defaultBranch) setTargetBranch(defaultBranch);
  }, [defaultBranch, setTargetBranch]);
  useEffect(() => setNewComment(""), [selectedIID, setNewComment]);

  const queryErrors = [mergeRequestsQuery, diffQuery, checksQuery, participantsQuery, commentsQuery, discussionsQuery, approvalsQuery]
    .map((query) => query.query.error);
  useEffect(() => {
    for (const error of queryErrors) {
      if (error) onError(extractErrorMessage(error));
    }
  }, [
    onError,
    mergeRequestsQuery.query.error,
    diffQuery.query.error,
    checksQuery.query.error,
    participantsQuery.query.error,
    commentsQuery.query.error,
    discussionsQuery.query.error,
    approvalsQuery.query.error,
  ]);

  return { loadMergeRequests, loadDiff, loadChecks, loadParticipants, loadComments, loadDiscussions, loadApprovals };
};
