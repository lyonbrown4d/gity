package mergerequest

import (
	"context"

	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

func (e *Endpoint) listMergeRequests(ctx context.Context, in *mergeRequestsInput) (*mergeRequestOutput, error) {
	items, err := e.service.List(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: items}, nil
}

func (e *Endpoint) getMergeRequest(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.GetByIID(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) getDiff(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.GetDiff(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) getChecks(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.GetChecks(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) listParticipants(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.ListParticipants(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) listComments(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.ListComments(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) listApprovals(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
	item, err := e.service.ListApprovals(ctx, in.ProjectID, in.MergeIID)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) createMergeRequest(ctx context.Context, in *createMergeRequestInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.CreateInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	authorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.AuthorUserID)
	if err != nil {
		return nil, err
	}
	input.AuthorUserID = authorUserID
	item, err := e.service.Create(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) createComment(ctx context.Context, in *createMergeRequestCommentInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.CommentInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	authorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.AuthorUserID)
	if err != nil {
		return nil, err
	}
	input.AuthorUserID = authorUserID
	if input.Body == "" {
		input.Body = in.Body.Content
	}
	item, err := e.service.CreateComment(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) approve(ctx context.Context, in *mergeRequestApprovalInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.ApprovalInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.UserID)
	if err != nil {
		return nil, err
	}
	input.UserID = userID
	item, err := e.service.Approve(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) unapprove(ctx context.Context, in *mergeRequestApprovalInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.ApprovalInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.UserID)
	if err != nil {
		return nil, err
	}
	input.UserID = userID
	item, err := e.service.Unapprove(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) mergeMergeRequest(ctx context.Context, in *mergeMergeRequestInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.MergeInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	actorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.ActorUserID)
	if err != nil {
		return nil, err
	}
	input.ActorUserID = actorUserID
	item, err := e.service.Merge(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) setReviewers(ctx context.Context, in *setParticipantsInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.ParticipantsInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.SetReviewers(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) setAssignees(ctx context.Context, in *setParticipantsInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.ParticipantsInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.SetAssignees(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}

func (e *Endpoint) updateMergeRequest(ctx context.Context, in *updateMergeRequestInput) (*mergeRequestOutput, error) {
	input, err := mapperx.MapStrict[mergerequestservice.UpdateInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.Update(ctx, in.ProjectID, in.MergeIID, input)
	if err != nil {
		return nil, err
	}
	return &mergeRequestOutput{Body: item}, nil
}
