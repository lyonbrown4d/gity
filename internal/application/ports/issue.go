package ports

import (
	"context"

	collectionx "github.com/arcgolabs/collectionx/list"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
)

type ProjectIssueRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[issuedomain.ProjectIssue], error)
	GetByProjectAndIID(ctx context.Context, projectID, iid int64) (issuedomain.ProjectIssue, error)
	Create(ctx context.Context, input CreateProjectIssueInput) (issuedomain.ProjectIssue, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectIssueInput) error
}

type ProjectIssueCommentRepository interface {
	ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueComment], error)
	Create(ctx context.Context, input CreateProjectIssueCommentInput) (issuedomain.ProjectIssueComment, error)
}

type ProjectIssueAssigneeRepository interface {
	ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error)
	ReplaceByIssueID(ctx context.Context, issueID int64, userIDs []int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error)
}

type ProjectIssueLabelRepository interface {
	ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueLabel], error)
	ReplaceByIssueID(ctx context.Context, issueID int64, labels []ProjectIssueLabelInput) (*collectionx.List[issuedomain.ProjectIssueLabel], error)
}

type ProjectIssueAttachmentRepository interface {
	ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueAttachment], error)
	GetByIssueAndID(ctx context.Context, issueID, attachmentID int64) (issuedomain.ProjectIssueAttachment, error)
	Create(ctx context.Context, input CreateProjectIssueAttachmentInput) (issuedomain.ProjectIssueAttachment, error)
	MarkStored(ctx context.Context, attachmentID int64, input StoreProjectIssueAttachmentInput) error
	DeleteByID(ctx context.Context, attachmentID int64) error
}

type CreateProjectIssueInput struct {
	ProjectID    int64
	AuthorUserID int64
	Title        string
	Description  string
	State        string
}

type UpdateProjectIssueInput struct {
	Title       *string
	Description *string
	State       *string
}

type CreateProjectIssueCommentInput struct {
	ProjectIssueID int64
	AuthorUserID   int64
	Body           string
}

type ProjectIssueLabelInput struct {
	Name  string
	Color string
}

type CreateProjectIssueAttachmentInput struct {
	ProjectIssueID   int64
	UploadedByUserID int64
	FileName         string
	ContentType      string
}

type StoreProjectIssueAttachmentInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}
