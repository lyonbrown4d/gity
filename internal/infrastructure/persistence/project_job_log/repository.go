package projectjoblog

import (
	"context"
	"fmt"
	ciports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJobLog, dbschema.ProjectJobLogSchemaDef]
}

type CreateInput = ciports.CreateProjectJobLogInput

type AppendInput = ciports.AppendProjectJobLogInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectJobLog](db, dbschema.ProjectJobLogSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectJobLogRepository(repo *Repository) ciports.ProjectJobLogRepository {
	return repo
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID, projectJobID int64) (*collectionx.List[cidomain.ProjectJobLog], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobLogSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID)).
		OrderBy(dbschema.ProjectJobLogSchema.Attempt.Asc(), dbschema.ProjectJobLogSchema.ID.Asc()).
		List(ctx))
}

func (r *Repository) LatestByProjectJobID(ctx context.Context, projectID, projectJobID int64) (cidomain.ProjectJobLog, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobLogSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID)).
		OrderBy(dbschema.ProjectJobLogSchema.Attempt.Desc(), dbschema.ProjectJobLogSchema.ID.Desc()).
		First(ctx))
}

func (r *Repository) GetByProjectJobAttempt(ctx context.Context, projectID, projectJobID int64, attempt int) (cidomain.ProjectJobLog, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobLogSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobLogSchema.ProjectJobID.Eq(projectJobID)).
		Where(dbschema.ProjectJobLogSchema.Attempt.Eq(attempt)).
		OrderBy(dbschema.ProjectJobLogSchema.ID.Desc()).
		First(ctx))
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
		if persistence.IsNotFound(err) {
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
		if persistence.IsNotFound(err) {
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
	if _, err := dbxrepo.PatchSet(r.base, projectJobLogKey(item.ID)).Set(
		dbschema.ProjectJobLogSchema.ExitCode.Set(item.ExitCode),
		dbschema.ProjectJobLogSchema.Output.Set(output),
		dbschema.ProjectJobLogSchema.OutputTruncated.Set(item.OutputTruncated),
		dbschema.ProjectJobLogSchema.DurationMillis.Set(item.DurationMillis),
		dbschema.ProjectJobLogSchema.UpdatedAt.Set(now),
	).Apply(ctx); err != nil {
		return cidomain.ProjectJobLog{}, fmt.Errorf("update project job log: %w", err)
	}
	item.Output = output
	item.UpdatedAt = now
	return item, nil
}

func projectJobLogKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectJobLogSchema.ID, id))
}
