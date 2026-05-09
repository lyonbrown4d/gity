package issue

import (
	"context"

	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	collectionlist "github.com/arcgolabs/collectionx/list"
)

func (e *Endpoint) listIssues(ctx context.Context, in *projectIssuesInput) (*issueOutput, error) {
	items, err := e.service.ListIssues(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item issuedomain.ProjectIssue) issueView {
		return toIssueView(item)
	}).Values()
	return &issueOutput{Body: views}, nil
}

func (e *Endpoint) getIssue(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
	item, err := e.service.GetIssueByIID(ctx, in.ProjectID, in.IssueIID)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: toIssueView(item)}, nil
}

func (e *Endpoint) createIssue(ctx context.Context, in *createIssueInput) (*issueOutput, error) {
	input, err := mapperx.MapStrict[issueservice.CreateIssueInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	authorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.AuthorUserID)
	if err != nil {
		return nil, err
	}
	input.AuthorUserID = authorUserID
	item, err := e.service.CreateIssue(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: toIssueView(item)}, nil
}

func (e *Endpoint) updateIssue(ctx context.Context, in *updateIssueInput) (*issueOutput, error) {
	input, err := mapperx.MapStrict[issueservice.UpdateIssueInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	if input.State == nil && in.Body.Status != nil {
		mapped := statusToState(*in.Body.Status)
		input.State = &mapped
	}
	item, err := e.service.UpdateIssue(ctx, in.ProjectID, in.IssueIID, input)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: toIssueView(item)}, nil
}

func (e *Endpoint) listComments(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
	items, err := e.service.ListComments(ctx, in.ProjectID, in.IssueIID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item issuedomain.ProjectIssueComment) issueCommentView {
		return toIssueCommentView(in.IssueIID, item)
	}).Values()
	return &issueOutput{Body: views}, nil
}

func (e *Endpoint) createComment(ctx context.Context, in *createCommentInput) (*issueOutput, error) {
	input, err := mapperx.MapStrict[issueservice.CreateCommentInput](e.mapper, in.Body)
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
	item, err := e.service.CreateComment(ctx, in.ProjectID, in.IssueIID, input)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: toIssueCommentView(in.IssueIID, item)}, nil
}

func (e *Endpoint) listAttachments(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
	items, err := e.service.ListAttachments(ctx, in.ProjectID, in.IssueIID)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: items}, nil
}

func (e *Endpoint) createAttachment(ctx context.Context, in *createAttachmentInput) (*issueOutput, error) {
	input, err := mapperx.MapStrict[issueservice.CreateAttachmentInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	uploadedByUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.UploadedByUserID)
	if err != nil {
		return nil, err
	}
	input.UploadedByUserID = uploadedByUserID
	item, err := e.service.CreateAttachment(ctx, in.ProjectID, in.IssueIID, input)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: item}, nil
}

func (e *Endpoint) getAttachment(ctx context.Context, in *projectAttachmentInput) (*issueOutput, error) {
	item, err := e.service.GetAttachmentContent(ctx, in.ProjectID, in.IssueIID, in.AttachmentID)
	if err != nil {
		return nil, err
	}
	return &issueOutput{Body: item}, nil
}
