package projectjoblog

import (
	"context"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJobLog, cidomain.ProjectJobLogSchemaDef]
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
		base: dbxrepo.NewWithOptions[cidomain.ProjectJobLog](db, cidomain.ProjectJobLogSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[cidomain.ProjectJobLog], error) {
	query := querydsl.Select(cidomain.ProjectJobLogSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobLogSchema).
		Where(querydsl.And(
			cidomain.ProjectJobLogSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(cidomain.ProjectJobLogSchema.Attempt.Asc(), cidomain.ProjectJobLogSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) LatestByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (cidomain.ProjectJobLog, error) {
	query := querydsl.Select(cidomain.ProjectJobLogSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobLogSchema).
		Where(querydsl.And(
			cidomain.ProjectJobLogSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(cidomain.ProjectJobLogSchema.Attempt.Desc(), cidomain.ProjectJobLogSchema.ID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectJobAttempt(ctx context.Context, projectID int64, projectJobID int64, attempt int) (cidomain.ProjectJobLog, error) {
	query := querydsl.Select(cidomain.ProjectJobLogSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobLogSchema).
		Where(querydsl.And(
			cidomain.ProjectJobLogSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID),
			cidomain.ProjectJobLogSchema.Attempt.Eq(attempt),
		)).
		OrderBy(cidomain.ProjectJobLogSchema.ID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectJobLog, error) {
	now := time.Now().UTC()
	truncated := 0
	if input.OutputTruncated {
		truncated = 1
	}
	item := cidomain.ProjectJobLog{
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
		return cidomain.ProjectJobLog{}, fmt.Errorf("insert project job log: %w", err)
	}
	return item, nil
}

func (r *Repository) Append(ctx context.Context, input AppendInput) (cidomain.ProjectJobLog, error) {
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
		return cidomain.ProjectJobLog{}, err
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

func (r *Repository) UpsertAttempt(ctx context.Context, input CreateInput) (cidomain.ProjectJobLog, error) {
	item, err := r.GetByProjectJobAttempt(ctx, input.ProjectID, input.ProjectJobID, input.Attempt)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return r.Create(ctx, input)
		}
		return cidomain.ProjectJobLog{}, err
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

func (r *Repository) update(ctx context.Context, item cidomain.ProjectJobLog) (cidomain.ProjectJobLog, error) {
	now := time.Now().UTC()
	output := strings.TrimRight(item.Output, "\r\n")
	if _, err := r.base.UpdateByID(ctx, item.ID,
		cidomain.ProjectJobLogSchema.ExitCode.Set(item.ExitCode),
		cidomain.ProjectJobLogSchema.Output.Set(output),
		cidomain.ProjectJobLogSchema.OutputTruncated.Set(item.OutputTruncated),
		cidomain.ProjectJobLogSchema.DurationMillis.Set(item.DurationMillis),
		cidomain.ProjectJobLogSchema.UpdatedAt.Set(now),
	); err != nil {
		return cidomain.ProjectJobLog{}, fmt.Errorf("update project job log: %w", err)
	}
	item.Output = output
	item.UpdatedAt = now
	return item, nil
}
