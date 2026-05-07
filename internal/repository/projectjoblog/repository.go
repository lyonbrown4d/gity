package projectjoblog

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/entity"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectJobLog, entity.ProjectJobLogSchemaDef]
}

type CreateInput struct {
	ProjectID       int64
	ProjectJobID    int64
	Attempt         int
	ExitCode        int
	Output          string
	OutputTruncated bool
	DurationMillis  int64
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectJobLog](db, entity.ProjectJobLogSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[entity.ProjectJobLog], error) {
	query := dbx.Select(entity.ProjectJobLogSchema.AllColumns().Values()...).
		From(entity.ProjectJobLogSchema).
		Where(dbx.And(
			entity.ProjectJobLogSchema.ProjectID.Eq(projectID),
			entity.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(entity.ProjectJobLogSchema.Attempt.Asc(), entity.ProjectJobLogSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) LatestByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (entity.ProjectJobLog, error) {
	query := dbx.Select(entity.ProjectJobLogSchema.AllColumns().Values()...).
		From(entity.ProjectJobLogSchema).
		Where(dbx.And(
			entity.ProjectJobLogSchema.ProjectID.Eq(projectID),
			entity.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(entity.ProjectJobLogSchema.Attempt.Desc(), entity.ProjectJobLogSchema.ID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectJobLog, error) {
	now := time.Now().UTC()
	truncated := 0
	if input.OutputTruncated {
		truncated = 1
	}
	item := entity.ProjectJobLog{
		ProjectID:       input.ProjectID,
		ProjectJobID:    input.ProjectJobID,
		Attempt:         input.Attempt,
		ExitCode:        input.ExitCode,
		Output:          strings.TrimRight(input.Output, "\r\n"),
		OutputTruncated: truncated,
		DurationMillis:  input.DurationMillis,
		CreatedAt:       now,
		UpdatedAt:       now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectJobLog{}, fmt.Errorf("insert project job log: %w", err)
	}
	return item, nil
}
