package projectissuecomment

import (
	"context"
	"fmt"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueComment, issuedomain.ProjectIssueCommentSchemaDef]
}

type CreateInput struct {
	ProjectIssueID int64
	AuthorUserID   int64
	Body           string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueComment](db, issuedomain.ProjectIssueCommentSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueComment], error) {
	query := querydsl.Select(issuedomain.ProjectIssueCommentSchema.AllColumns().Values()...).
		From(issuedomain.ProjectIssueCommentSchema).
		Where(issuedomain.ProjectIssueCommentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(issuedomain.ProjectIssueCommentSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (issuedomain.ProjectIssueComment, error) {
	now := time.Now().UTC()
	item := issuedomain.ProjectIssueComment{
		ProjectIssueID: input.ProjectIssueID,
		AuthorUserID:   input.AuthorUserID,
		Body:           strings.TrimSpace(input.Body),
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return issuedomain.ProjectIssueComment{}, fmt.Errorf("insert project issue comment: %w", err)
	}
	return item, nil
}
