package projectissuecomment

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectIssueComment, entity.ProjectIssueCommentSchemaDef]
}

type CreateInput struct {
	ProjectIssueID int64
	AuthorUserID   int64
	Body           string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectIssueComment](db, entity.ProjectIssueCommentSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[entity.ProjectIssueComment], error) {
	query := querydsl.Select(entity.ProjectIssueCommentSchema.AllColumns().Values()...).
		From(entity.ProjectIssueCommentSchema).
		Where(entity.ProjectIssueCommentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(entity.ProjectIssueCommentSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectIssueComment, error) {
	now := time.Now().UTC()
	item := entity.ProjectIssueComment{
		ProjectIssueID: input.ProjectIssueID,
		AuthorUserID:   input.AuthorUserID,
		Body:           strings.TrimSpace(input.Body),
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectIssueComment{}, fmt.Errorf("insert project issue comment: %w", err)
	}
	return item, nil
}
