package ports

import (
	"context"

	collectionx "github.com/arcgolabs/collectionx/list"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

type ProjectMergeRequestRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequest], error)
	GetByProjectAndIID(ctx context.Context, projectID, iid int64) (mergedomain.ProjectMergeRequest, error)
	Create(ctx context.Context, input CreateProjectMergeRequestInput) (mergedomain.ProjectMergeRequest, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectMergeRequestInput) error
}

type ProjectMergeRequestParticipantRepository interface {
	ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestParticipant], error)
	ReplaceByMergeRequestAndRole(ctx context.Context, mergeRequestID int64, role string, userIDs []int64) (*collectionx.List[mergedomain.ProjectMergeRequestParticipant], error)
}

type ProjectMergeRequestCommentRepository interface {
	ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestComment], error)
	Create(ctx context.Context, input CreateProjectMergeRequestCommentInput) (mergedomain.ProjectMergeRequestComment, error)
}

type ProjectMergeRequestApprovalRepository interface {
	ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestApproval], error)
	Upsert(ctx context.Context, input UpsertProjectMergeRequestApprovalInput) (mergedomain.ProjectMergeRequestApproval, error)
	DeleteByMergeRequestAndUser(ctx context.Context, mergeRequestID, userID int64) error
}

type ProjectMergeRequestApprovalRuleRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequestApprovalRule], error)
	GetByProjectAndID(ctx context.Context, projectID, id int64) (mergedomain.ProjectMergeRequestApprovalRule, error)
	Create(ctx context.Context, input CreateProjectMergeRequestApprovalRuleInput) (mergedomain.ProjectMergeRequestApprovalRule, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectMergeRequestApprovalRuleInput) error
	DeleteByProjectAndID(ctx context.Context, projectID, id int64) error
}

type CreateProjectMergeRequestInput struct {
	ProjectID    int64
	AuthorUserID int64
	Title        string
	Description  string
	SourceBranch string
	TargetBranch string
}

type UpdateProjectMergeRequestInput struct {
	Title       *string
	Description *string
	State       *string
}

type CreateProjectMergeRequestCommentInput struct {
	MergeRequestID int64
	AuthorUserID   int64
	Body           string
}

type UpsertProjectMergeRequestApprovalInput struct {
	MergeRequestID int64
	UserID         int64
}

type CreateProjectMergeRequestApprovalRuleInput struct {
	ProjectID         int64
	Name              string
	TargetBranch      string
	ApprovalsRequired int
	EligibleUserIDs   string
	CodeOwner         int
}

type UpdateProjectMergeRequestApprovalRuleInput struct {
	Name              *string
	TargetBranch      *string
	ApprovalsRequired *int
	EligibleUserIDs   *string
	CodeOwner         *int
}
