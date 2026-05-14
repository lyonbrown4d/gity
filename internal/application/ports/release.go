package ports

import (
	"context"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
)

type ProjectReleaseRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[releasedomain.ProjectRelease], error)
	GetByID(ctx context.Context, id int64) (releasedomain.ProjectRelease, error)
	GetByProjectAndTagName(ctx context.Context, projectID int64, tagName string) (releasedomain.ProjectRelease, error)
	Create(ctx context.Context, input CreateProjectReleaseInput) (releasedomain.ProjectRelease, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectReleaseInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type ProjectReleaseLinkRepository interface {
	ListByReleaseID(ctx context.Context, releaseID int64) (*collectionx.List[releasedomain.ProjectReleaseLink], error)
	ListByReleaseIDs(ctx context.Context, releaseIDs ...int64) (*collectionx.List[releasedomain.ProjectReleaseLink], error)
	Create(ctx context.Context, input CreateProjectReleaseLinkInput) (releasedomain.ProjectReleaseLink, error)
	DeleteByID(ctx context.Context, id int64) error
}

type CreateProjectReleaseInput struct {
	ProjectID       int64
	TagName         string
	Name            string
	Description     string
	CreatedByUserID int64
	ReleasedAt      time.Time
}

type UpdateProjectReleaseInput struct {
	Name        *string
	Description *string
	ReleasedAt  *time.Time
}

type CreateProjectReleaseLinkInput struct {
	ProjectReleaseID int64
	Name             string
	URL              string
	LinkType         string
}
