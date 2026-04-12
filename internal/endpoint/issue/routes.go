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
	ProjectID int64           `path:"id"`
	Body      createIssueBody `json:"body"`
}

type updateIssueInput struct {
	ProjectID int64           `path:"id"`
	IssueIID  int64           `path:"issue_iid"`
	Body      updateIssueBody `json:"body"`
}

type createCommentInput struct {
	ProjectID int64             `path:"id"`
	IssueIID  int64             `path:"issue_iid"`
	Body      createCommentBody `json:"body"`
}

type createAttachmentInput struct {
	ProjectID int64                `path:"id"`
	IssueIID  int64                `path:"issue_iid"`
	Body      createAttachmentBody `json:"body"`
}

type issueOutput struct {
	Body any `json:"body"`
}

type createIssueBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
}

type updateIssueBody struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
}

type createCommentBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Body         string `json:"body"`
}

type createAttachmentBody struct {
	UploadedByUserID int64  `json:"uploaded_by_user_id"`
	FileName         string `json:"file_name"`
	ContentType      string `json:"content_type"`
	ContentBase64    string `json:"content_base64"`
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
		item, err := service.CreateIssue(ctx, in.ProjectID, issueservice.CreateIssueInput{
			AuthorUserID: in.Body.AuthorUserID,
			Title:        in.Body.Title,
			Description:  in.Body.Description,
		})
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	})

	httpx.MustGroupPatch(v1, "/projects/{id}/issues/{issue_iid}", func(ctx context.Context, in *updateIssueInput) (*issueOutput, error) {
		item, err := service.UpdateIssue(ctx, in.ProjectID, in.IssueIID, issueservice.UpdateIssueInput{
			Title:       in.Body.Title,
			Description: in.Body.Description,
			State:       in.Body.State,
		})
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
		item, err := service.CreateComment(ctx, in.ProjectID, in.IssueIID, issueservice.CreateCommentInput{
			AuthorUserID: in.Body.AuthorUserID,
			Body:         in.Body.Body,
		})
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
		item, err := service.CreateAttachment(ctx, in.ProjectID, in.IssueIID, issueservice.CreateAttachmentInput{
			UploadedByUserID: in.Body.UploadedByUserID,
			FileName:         in.Body.FileName,
			ContentType:      in.Body.ContentType,
			ContentBase64:    in.Body.ContentBase64,
		})
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
