import type {
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
import {
  isRecord,
  normalizeBoolean as normalizeBool,
  normalizeNumber,
  normalizeOptionalString,
  normalizeString,
  normalizeStringArray,
  resolveArrayPayload,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "../repository-normalizers";

export const resolveMergeRequestList = (payload: unknown): RawRecord[] => resolveArrayPayload(payload).filter(isRecord);

export const normalizeDiffView = (payload: unknown): RepositoryMergeRequestDiffView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    diff: normalizeString(raw.diff ?? raw.Diff),
  };
};

export const normalizeCheckStatusView = (payload: unknown): RepositoryMergeRequestCheckStatusView | null => {
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

export const normalizeParticipantsView = (payload: unknown): RepositoryMergeRequestParticipantsView | null => {
  const raw = isRecord(payload) ? (payload.body ?? payload.Body ?? payload) : null;
  if (!isRecord(raw)) {
    return null;
  }
  return {
    merge_request: normalizeMergeRequest(raw.merge_request ?? raw.MergeRequest),
    participants: resolveRecordArray(raw.participants ?? raw.Participants).map(normalizeParticipant),
  };
};

export const normalizeCommentsView = (payload: unknown): RepositoryMergeRequestCommentsView | null => {
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

export const normalizeApprovalsView = (payload: unknown): RepositoryMergeRequestApprovalsView | null => {
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

export const normalizeMergeRequest = (rawValue: unknown): RepositoryMergeRequestView => {
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

export const resolveUserList = (payload: unknown): RawRecord[] =>
  resolveArrayPayload<RawRecord>(payload).filter(isRecord);

export const normalizeUser = (raw: RawRecord): UserView => ({
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
