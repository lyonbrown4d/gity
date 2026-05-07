package ports

import (
	"context"

	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type ProjectMergeRequestRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequest], error)
	GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (mergedomain.ProjectMergeRequest, error)
	Create(ctx context.Context, input CreateProjectMergeRequestInput) (mergedomain.ProjectMergeRequest, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectMergeRequestInput) error
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
