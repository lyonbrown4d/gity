package issue

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	issueservice "github.com/DaiYuANg/gity/internal/service/issue"
)

type projectIssueInput struct {
	ProjectID int64 `path:"id"`
	IssueIID  int64 `path:"issue_iid"`
}

type projectAttachmentInput struct {
	ProjectID    int64 `path:"id"`
	IssueIID     int64 `path:"issue_iid"`
	AttachmentID int64 `path:"attachment_id"`
}

type createIssueInput struct {
	ProjectID int64                         `path:"id"`
	Body      issueservice.CreateIssueInput `json:"body"`
}

type updateIssueInput struct {
	ProjectID int64                         `path:"id"`
	IssueIID  int64                         `path:"issue_iid"`
	Body      issueservice.UpdateIssueInput `json:"body"`
}

type createCommentInput struct {
	ProjectID int64                           `path:"id"`
	IssueIID  int64                           `path:"issue_iid"`
	Body      issueservice.CreateCommentInput `json:"body"`
}

type createAttachmentInput struct {
	ProjectID int64                              `path:"id"`
	IssueIID  int64                              `path:"issue_iid"`
	Body      issueservice.CreateAttachmentInput `json:"body"`
}

type issueOutput struct {
	Body any `json:"body"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *issueservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/projects/{id}/issues", func(ctx context.Context, in *struct {
		ProjectID int64 `path:"id"`
	}) (*issueOutput, error) {
		items, err := service.ListIssues(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/issues/{issue_iid}", func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		item, err := service.GetIssueByIID(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/issues", func(ctx context.Context, in *createIssueInput) (*issueOutput, error) {
		item, err := service.CreateIssue(ctx, in.ProjectID, in.Body)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupPatch(v1, "/projects/{id}/issues/{issue_iid}", func(ctx context.Context, in *updateIssueInput) (*issueOutput, error) {
		item, err := service.UpdateIssue(ctx, in.ProjectID, in.IssueIID, in.Body)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/issues/{issue_iid}/comments", func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListComments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: items}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/issues/{issue_iid}/comments", func(ctx context.Context, in *createCommentInput) (*issueOutput, error) {
		item, err := service.CreateComment(ctx, in.ProjectID, in.IssueIID, in.Body)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/issues/{issue_iid}/attachments", func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListAttachments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: items}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/issues/{issue_iid}/attachments", func(ctx context.Context, in *createAttachmentInput) (*issueOutput, error) {
		item, err := service.CreateAttachment(ctx, in.ProjectID, in.IssueIID, in.Body)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}", func(ctx context.Context, in *projectAttachmentInput) (*issueOutput, error) {
		item, err := service.GetAttachmentContent(ctx, in.ProjectID, in.IssueIID, in.AttachmentID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})
}
