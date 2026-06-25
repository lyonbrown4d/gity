package mergerequest

import (
	"strconv"
	"time"

	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

func toMergeRequestViews(items []mergedomain.ProjectMergeRequest) []mergeRequestView {
	views := make([]mergeRequestView, 0, len(items))
	for index := range items {
		views = append(views, toMergeRequestView(items[index]))
	}
	return views
}

func toMergeRequestView(item mergedomain.ProjectMergeRequest) mergeRequestView {
	return mergeRequestView{
		ID:           formatID(item.ID),
		ProjectID:    formatID(item.ProjectID),
		IID:          item.IID,
		AuthorUserID: formatID(item.AuthorUserID),
		Title:        item.Title,
		Description:  item.Description,
		State:        item.State,
		SourceBranch: item.SourceBranch,
		TargetBranch: item.TargetBranch,
		CreatedAt:    formatMergeRequestTime(item.CreatedAt),
		UpdatedAt:    formatMergeRequestTime(item.UpdatedAt),
	}
}

func toMergeRequestDiffView(item mergerequestservice.DiffView) mergeRequestDiffView {
	return mergeRequestDiffView{
		MergeRequest: toMergeRequestView(item.MergeRequest),
		Diff:         item.Diff,
	}
}

func toMergeRequestCheckStatusView(item mergerequestservice.CheckStatusView) mergeRequestCheckStatusView {
	return mergeRequestCheckStatusView{
		MergeRequest:           toMergeRequestView(item.MergeRequest),
		SourceBranch:           item.SourceBranch,
		SourceCommitSHA:        item.SourceCommitSHA,
		TargetBranch:           item.TargetBranch,
		TargetBranchProtected:  item.TargetBranchProtected,
		RequireMergeRequest:    item.RequireMergeRequest,
		RequirePipelineSuccess: item.RequirePipelineSuccess,
		RequireApproval:        item.RequireApproval,
		RequiredApprovals:      item.RequiredApprovals,
		ApprovalCount:          item.ApprovalCount,
		ApprovalRules:          toMergeRequestApprovalRuleCheckViews(item.ApprovalRules),
		MergeAccessLevel:       item.MergeAccessLevel,
		PipelineRequired:       item.PipelineRequired,
		Required:               item.Required,
		Mergeable:              item.Mergeable,
		Status:                 item.Status,
		BlockingReason:         item.BlockingReason,
		Blockers:               toMergeRequestCheckBlockerViews(item.Blockers),
		Pipeline:               toMergeRequestPipelineView(item.Pipeline),
	}
}

func toMergeRequestCheckBlockerViews(items []mergerequestservice.CheckBlockerView) []mergeRequestCheckBlockerView {
	views := make([]mergeRequestCheckBlockerView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, mergeRequestCheckBlockerView{
			Code:     item.Code,
			Category: item.Category,
			Message:  item.Message,
		})
	}
	return views
}

func toMergeRequestPipelineView(item *cidomain.ProjectPipeline) *mergeRequestPipelineView {
	if item == nil {
		return nil
	}
	return &mergeRequestPipelineView{
		ID:            formatID(item.ID),
		ProjectID:     formatID(item.ProjectID),
		IID:           item.IID,
		Name:          item.Name,
		Source:        item.Source,
		RefName:       item.RefName,
		CommitSHA:     item.CommitSHA,
		Status:        item.Status,
		ConfigSource:  item.ConfigSource,
		ConfigContent: item.ConfigContent,
		CreatedAt:     formatMergeRequestTime(item.CreatedAt),
		UpdatedAt:     formatMergeRequestTime(item.UpdatedAt),
		StartedAt:     formatMergeRequestTime(item.StartedAt),
		FinishedAt:    formatMergeRequestTime(item.FinishedAt),
	}
}

func toMergeRequestParticipantsView(item mergerequestservice.ParticipantsView) mergeRequestParticipantsView {
	return mergeRequestParticipantsView{
		MergeRequest: toMergeRequestView(item.MergeRequest),
		Participants: toMergeRequestParticipantViews(item.Participants),
	}
}

func toMergeRequestParticipantViews(items []mergedomain.ProjectMergeRequestParticipant) []mergeRequestParticipantView {
	views := make([]mergeRequestParticipantView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, mergeRequestParticipantView{
			ID:             formatID(item.ID),
			MergeRequestID: formatID(item.MergeRequestID),
			UserID:         formatID(item.UserID),
			Role:           item.Role,
			CreatedAt:      formatMergeRequestTime(item.CreatedAt),
			UpdatedAt:      formatMergeRequestTime(item.UpdatedAt),
		})
	}
	return views
}

func toMergeRequestCommentsView(item mergerequestservice.CommentsView) mergeRequestCommentsView {
	return mergeRequestCommentsView{
		MergeRequest: toMergeRequestView(item.MergeRequest),
		Comments:     toMergeRequestCommentViews(item.Comments),
	}
}

func toMergeRequestCommentViews(items []mergedomain.ProjectMergeRequestComment) []mergeRequestCommentView {
	views := make([]mergeRequestCommentView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, mergeRequestCommentView{
			ID:             formatID(item.ID),
			MergeRequestID: formatID(item.MergeRequestID),
			AuthorUserID:   formatID(item.AuthorUserID),
			Body:           item.Body,
			CreatedAt:      formatMergeRequestTime(item.CreatedAt),
			UpdatedAt:      formatMergeRequestTime(item.UpdatedAt),
		})
	}
	return views
}

func toMergeRequestApprovalsView(item mergerequestservice.ApprovalsView) mergeRequestApprovalsView {
	return mergeRequestApprovalsView{
		MergeRequest: toMergeRequestView(item.MergeRequest),
		Approvals:    toMergeRequestApprovalViews(item.Approvals),
	}
}

func toMergeRequestApprovalViews(items []mergedomain.ProjectMergeRequestApproval) []mergeRequestApprovalView {
	views := make([]mergeRequestApprovalView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, mergeRequestApprovalView{
			ID:             formatID(item.ID),
			MergeRequestID: formatID(item.MergeRequestID),
			UserID:         formatID(item.UserID),
			CreatedAt:      formatMergeRequestTime(item.CreatedAt),
			UpdatedAt:      formatMergeRequestTime(item.UpdatedAt),
		})
	}
	return views
}

func toMergeRequestApprovalRulesView(item mergerequestservice.ApprovalRulesView) mergeRequestApprovalRulesView {
	return mergeRequestApprovalRulesView{
		ProjectID: formatID(item.ProjectID),
		Rules:     toMergeRequestApprovalRuleViews(item.Rules),
	}
}

func toMergeRequestApprovalRuleViews(items []mergerequestservice.ApprovalRuleView) []mergeRequestApprovalRuleView {
	views := make([]mergeRequestApprovalRuleView, 0, len(items))
	for index := range items {
		views = append(views, toMergeRequestApprovalRuleView(items[index]))
	}
	return views
}

func toMergeRequestApprovalRuleView(item mergerequestservice.ApprovalRuleView) mergeRequestApprovalRuleView {
	return mergeRequestApprovalRuleView{
		ID:                formatID(item.ID),
		ProjectID:         formatID(item.ProjectID),
		Name:              item.Name,
		TargetBranch:      item.TargetBranch,
		ApprovalsRequired: item.ApprovalsRequired,
		EligibleUserIDs:   formatIDs(item.EligibleUserIDs),
		CodeOwner:         item.CodeOwner,
	}
}

func toMergeRequestApprovalRuleCheckViews(items []mergerequestservice.ApprovalRuleCheck) []mergeRequestApprovalRuleCheckView {
	views := make([]mergeRequestApprovalRuleCheckView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, mergeRequestApprovalRuleCheckView{
			RuleID:            formatID(item.RuleID),
			Name:              item.Name,
			TargetBranch:      item.TargetBranch,
			ApprovalsRequired: item.ApprovalsRequired,
			ApprovalCount:     item.ApprovalCount,
			EligibleUserIDs:   formatIDs(item.EligibleUserIDs),
			CodeOwner:         item.CodeOwner,
			Satisfied:         item.Satisfied,
			BlockingReason:    item.BlockingReason,
		})
	}
	return views
}

func formatIDs(values []int64) []string {
	ids := make([]string, 0, len(values))
	for _, value := range values {
		ids = append(ids, formatID(value))
	}
	return ids
}

func formatID(value int64) string {
	return strconv.FormatInt(value, 10)
}

func formatMergeRequestTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}
