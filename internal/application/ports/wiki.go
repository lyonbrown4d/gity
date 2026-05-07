package ports

import (
	"context"

	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type ProjectWikiPageRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[wikidomain.ProjectWikiPage], error)
	GetByProjectAndSlug(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error)
	Create(ctx context.Context, input CreateProjectWikiPageInput) (wikidomain.ProjectWikiPage, error)
	UpdateByID(ctx context.Context, id int64, input UpdateProjectWikiPageInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type CreateProjectWikiPageInput struct {
	ProjectID          int64
	Slug               string
	Title              string
	Content            string
	Format             string
	AuthorUserID       int64
	LastEditedByUserID int64
}

type UpdateProjectWikiPageInput struct {
	Title              *string
	Content            *string
	LastEditedByUserID int64
}
