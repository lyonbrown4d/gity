import { useCustomMutation } from "@refinedev/core";
import type {
  RepositoryMergeRequestParticipantRole,
  RepositoryMergeRequestState,
  RepositoryMergeRequestView,
} from "@/pages/types";
import { extractErrorMessage } from "../issues-utils";
import { normalizeMergeRequest } from "./data";
import type { InlineReviewPosition } from "./diff-review";
import type { RawRecord } from "../repository-normalizers";

interface UseMergeRequestActionsArgs {
  repoId: string;
  defaultBranch: string;
  selectedMergeRequest: RepositoryMergeRequestView | null;
  title: string;
  description: string;
  sourceBranch: string;
  targetBranch: string;
  newComment: string;
  t: (text: string) => string;
  setSelectedIID: (iid: number) => void;
  setComposerOpen: (open: boolean) => void;
  setTitle: (value: string) => void;
  setDescription: (value: string) => void;
  setSourceBranch: (value: string) => void;
  setTargetBranch: (value: string) => void;
  setReviewerDraftUserID: (value: string) => void;
  setAssigneeDraftUserID: (value: string) => void;
  setNewComment: (value: string) => void;
  onError: (message: string | null) => void;
  onMerged: () => Promise<void>;
  loadMergeRequests: () => Promise<void>;
  loadDiff: () => Promise<void>;
  loadChecks: () => Promise<void>;
  loadParticipants: () => Promise<void>;
  loadComments: () => Promise<void>;
  loadDiscussions: () => Promise<void>;
  loadApprovals: () => Promise<void>;
}

export const useMergeRequestActions = ({
  repoId, defaultBranch, selectedMergeRequest, title, description, t,
  sourceBranch, targetBranch, newComment, setSelectedIID, setComposerOpen, setTitle,
  setDescription, setSourceBranch, setTargetBranch, setReviewerDraftUserID,
  setAssigneeDraftUserID, setNewComment, onError, onMerged, loadMergeRequests,
  loadDiff, loadChecks, loadParticipants, loadComments, loadDiscussions, loadApprovals,
}: UseMergeRequestActionsArgs) => {
  const { mutateAsync: createMergeRequest, mutation: { isPending: isCreating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: updateMergeRequest, mutation: { isPending: isUpdating } } = useCustomMutation<RawRecord>();
  const { mutateAsync: mergeMergeRequest, mutation: { isPending: isMerging } } = useCustomMutation<RawRecord>();
  const { mutateAsync: setMergeRequestParticipants, mutation: { isPending: isUpdatingParticipants } } = useCustomMutation<RawRecord>();
  const { mutateAsync: createMergeRequestComment, mutation: { isPending: isCreatingComment } } = useCustomMutation<RawRecord>();
  const { mutateAsync: approveMergeRequest, mutation: { isPending: isUpdatingApproval } } = useCustomMutation<RawRecord>();
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

  const submitInlineComment = async (body: string, position: InlineReviewPosition, discussionID?: string): Promise<boolean> => {
    if (!selectedMergeRequest || isCreatingComment) return false;
    onError(null);
    try {
      await createMergeRequestComment({
        url: `/projects/${repoId}/merge-requests/${selectedMergeRequest.iid}/comments`,
        method: "post",
        values: { body, position, ...(discussionID ? { discussion_id: discussionID } : {}) },
      });
      await loadDiscussions();
      return true;
    } catch (error) {
      onError(extractErrorMessage(error));
      return false;
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

  return {
    isCreating, isUpdating, isMerging, isUpdatingParticipants, isCreatingComment, isUpdatingApproval,
    submitCreateMergeRequest, submitUpdateState, submitMerge, submitSetParticipants,
    submitCreateComment, submitInlineComment, submitApproval,
  };
};
