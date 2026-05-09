package projectmergerequestcomment

import (
	"context"
	"strings"
	"time"

	mergeports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[mergedomain.ProjectMergeRequestComment, dbschema.ProjectMergeRequestCommentSchemaDef]
}

type CreateInput = mergeports.CreateProjectMergeRequestCommentInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequestComment](db, dbschema.ProjectMergeRequestCommentSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectMergeRequestCommentRepository(repo *Repository) mergeports.ProjectMergeRequestCommentRepository {
	return repo
}

func (r *Repository) ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestComment], error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestCommentSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestCommentSchema).
		Where(dbschema.ProjectMergeRequestCommentSchema.MergeRequestID.Eq(mergeRequestID)).
		OrderBy(dbschema.ProjectMergeRequestCommentSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (mergedomain.ProjectMergeRequestComment, error) {
	now := time.Now().UTC()
	item := mergedomain.ProjectMergeRequestComment{
		MergeRequestID: input.MergeRequestID,
		AuthorUserID:   input.AuthorUserID,
		Body:           strings.TrimSpace(input.Body),
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return mergedomain.ProjectMergeRequestComment{}, oops.In("persistence.merge_request_comment").
			With("merge_request_id", input.MergeRequestID, "author_user_id", input.AuthorUserID).
			Wrapf(err, "insert merge request comment")
	}
	return item, nil
}
