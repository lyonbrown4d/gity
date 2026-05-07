package projectjoblog

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

type AppendInput struct {
	ProjectID       int64
	ProjectJobID    int64
	Attempt         int
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
	query := querydsl.Select(entity.ProjectJobLogSchema.AllColumns().Values()...).
		From(entity.ProjectJobLogSchema).
		Where(querydsl.And(
			entity.ProjectJobLogSchema.ProjectID.Eq(projectID),
			entity.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(entity.ProjectJobLogSchema.Attempt.Asc(), entity.ProjectJobLogSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) LatestByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (entity.ProjectJobLog, error) {
	query := querydsl.Select(entity.ProjectJobLogSchema.AllColumns().Values()...).
		From(entity.ProjectJobLogSchema).
		Where(querydsl.And(
			entity.ProjectJobLogSchema.ProjectID.Eq(projectID),
			entity.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(entity.ProjectJobLogSchema.Attempt.Desc(), entity.ProjectJobLogSchema.ID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectJobAttempt(ctx context.Context, projectID int64, projectJobID int64, attempt int) (entity.ProjectJobLog, error) {
	query := querydsl.Select(entity.ProjectJobLogSchema.AllColumns().Values()...).
		From(entity.ProjectJobLogSchema).
		Where(querydsl.And(
			entity.ProjectJobLogSchema.ProjectID.Eq(projectID),
			entity.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
			entity.ProjectJobLogSchema.Attempt.Eq(attempt),
		)).
		OrderBy(entity.ProjectJobLogSchema.ID.Desc()).
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

func (r *Repository) Append(ctx context.Context, input AppendInput) (entity.ProjectJobLog, error) {
	item, err := r.GetByProjectJobAttempt(ctx, input.ProjectID, input.ProjectJobID, input.Attempt)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return r.Create(ctx, CreateInput{
				ProjectID:       input.ProjectID,
				ProjectJobID:    input.ProjectJobID,
				Attempt:         input.Attempt,
				Output:          input.Output,
				OutputTruncated: input.OutputTruncated,
				DurationMillis:  input.DurationMillis,
			})
		}
		return entity.ProjectJobLog{}, err
	}
	item.Output += input.Output
	if input.OutputTruncated {
		item.OutputTruncated = 1
	}
	if input.DurationMillis > item.DurationMillis {
		item.DurationMillis = input.DurationMillis
	}
	return r.update(ctx, item)
}

func (r *Repository) UpsertAttempt(ctx context.Context, input CreateInput) (entity.ProjectJobLog, error) {
	item, err := r.GetByProjectJobAttempt(ctx, input.ProjectID, input.ProjectJobID, input.Attempt)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return r.Create(ctx, input)
		}
		return entity.ProjectJobLog{}, err
	}
	item.ExitCode = input.ExitCode
	item.Output = strings.TrimRight(input.Output, "\r\n")
	if input.OutputTruncated {
		item.OutputTruncated = 1
	} else {
		item.OutputTruncated = 0
	}
	item.DurationMillis = input.DurationMillis
	return r.update(ctx, item)
}

func (r *Repository) update(ctx context.Context, item entity.ProjectJobLog) (entity.ProjectJobLog, error) {
	now := time.Now().UTC()
	output := strings.TrimRight(item.Output, "\r\n")
	if _, err := r.base.UpdateByID(ctx, item.ID,
		entity.ProjectJobLogSchema.ExitCode.Set(item.ExitCode),
		entity.ProjectJobLogSchema.Output.Set(output),
		entity.ProjectJobLogSchema.OutputTruncated.Set(item.OutputTruncated),
		entity.ProjectJobLogSchema.DurationMillis.Set(item.DurationMillis),
		entity.ProjectJobLogSchema.UpdatedAt.Set(now),
	); err != nil {
		return entity.ProjectJobLog{}, fmt.Errorf("update project job log: %w", err)
	}
	item.Output = output
	item.UpdatedAt = now
	return item, nil
}
