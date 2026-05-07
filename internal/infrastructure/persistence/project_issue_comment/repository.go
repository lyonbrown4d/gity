package projectissuecomment

import (
	"context"
	"fmt"
	issueports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueComment, dbschema.ProjectIssueCommentSchemaDef]
}

type CreateInput = issueports.CreateProjectIssueCommentInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueComment](db, dbschema.ProjectIssueCommentSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueCommentRepository(repo *Repository) issueports.ProjectIssueCommentRepository {
	return repo
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueComment], error) {
	query := querydsl.Select(dbschema.ProjectIssueCommentSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueCommentSchema).
		Where(dbschema.ProjectIssueCommentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueCommentSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
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
