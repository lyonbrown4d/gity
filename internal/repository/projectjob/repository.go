package projectjob

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

const (
	StatusPending   = "pending"
	StatusRunning   = "running"
	StatusSucceeded = "succeeded"
	StatusFailed    = "failed"
	StatusCancelled = "cancelled"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectJob, entity.ProjectJobSchemaDef]
}

type CreateInput struct {
	ProjectID   int64
	Kind        string
	Payload     string
	MaxAttempts int
	RunAfter    time.Time
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectJob](db, entity.ProjectJobSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[entity.ProjectJob], error) {
	query := dbx.Select(entity.ProjectJobSchema.AllColumns().Values()...).
		From(entity.ProjectJobSchema).
		Where(entity.ProjectJobSchema.ProjectID.Eq(projectID)).
		OrderBy(entity.ProjectJobSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (entity.ProjectJob, error) {
	query := dbx.Select(entity.ProjectJobSchema.AllColumns().Values()...).
		From(entity.ProjectJobSchema).
		Where(dbx.And(
			entity.ProjectJobSchema.ProjectID.Eq(projectID),
			entity.ProjectJobSchema.ID.Eq(id),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.ProjectJob, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectJob, error) {
	now := time.Now().UTC()
	runAfter := input.RunAfter.UTC()
	if runAfter.IsZero() {
		runAfter = now
	}
	maxAttempts := input.MaxAttempts
	if maxAttempts <= 0 {
		maxAttempts = 3
	}
	item := entity.ProjectJob{
		ProjectID:   input.ProjectID,
		Kind:        strings.TrimSpace(input.Kind),
		Status:      StatusPending,
		Payload:     strings.TrimSpace(input.Payload),
		Attempts:    0,
		MaxAttempts: maxAttempts,
		RunAfter:    runAfter,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectJob{}, fmt.Errorf("insert project job: %w", err)
	}
	return item, nil
}

func (r *Repository) ClaimNext(ctx context.Context, workerID string, lease time.Duration) (entity.ProjectJob, bool, error) {
	return r.claimNext(ctx, 0, nil, workerID, lease)
}

func (r *Repository) ClaimNextByKinds(ctx context.Context, kinds []string, workerID string, lease time.Duration) (entity.ProjectJob, bool, error) {
	return r.claimNext(ctx, 0, kinds, workerID, lease)
}

func (r *Repository) ClaimNextByProjectID(ctx context.Context, projectID int64, workerID string, lease time.Duration) (entity.ProjectJob, bool, error) {
	return r.claimNext(ctx, projectID, nil, workerID, lease)
}

func (r *Repository) ClaimNextByProjectIDAndKinds(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (entity.ProjectJob, bool, error) {
	return r.claimNext(ctx, projectID, kinds, workerID, lease)
}

func (r *Repository) claimNext(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (entity.ProjectJob, bool, error) {
	if lease <= 0 {
		lease = time.Minute
	}
	now := time.Now().UTC()
	predicates := []dbx.Predicate{
		entity.ProjectJobSchema.Status.Eq(StatusPending),
		entity.ProjectJobSchema.RunAfter.Le(now),
	}
	if projectID > 0 {
		predicates = append(predicates, entity.ProjectJobSchema.ProjectID.Eq(projectID))
	}
	if len(kinds) > 0 {
		predicates = append(predicates, entity.ProjectJobSchema.Kind.In(kinds...))
	}
	query := dbx.Select(entity.ProjectJobSchema.AllColumns().Values()...).
		From(entity.ProjectJobSchema).
		Where(dbx.And(predicates...)).
		OrderBy(entity.ProjectJobSchema.RunAfter.Asc(), entity.ProjectJobSchema.ID.Asc()).
		Limit(1)
	item, err := r.base.First(ctx, query)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return entity.ProjectJob{}, false, nil
		}
		return entity.ProjectJob{}, false, err
	}
	item.Status = StatusRunning
	item.Attempts++
	item.LockedBy = strings.TrimSpace(workerID)
	item.LockedUntil = now.Add(lease)
	item.StartedAt = now
	item.UpdatedAt = now
	if _, err := r.base.UpdateByID(ctx, item.ID,
		entity.ProjectJobSchema.Status.Set(item.Status),
		entity.ProjectJobSchema.Attempts.Set(item.Attempts),
		entity.ProjectJobSchema.LockedBy.Set(item.LockedBy),
		entity.ProjectJobSchema.LockedUntil.Set(item.LockedUntil),
		entity.ProjectJobSchema.StartedAt.Set(item.StartedAt),
		entity.ProjectJobSchema.UpdatedAt.Set(item.UpdatedAt),
	); err != nil {
		return entity.ProjectJob{}, false, fmt.Errorf("claim project job: %w", err)
	}
	return item, true, nil
}

func (r *Repository) MarkSucceeded(ctx context.Context, id int64, result string) error {
	now := time.Now().UTC()
	if _, err := r.base.UpdateByID(ctx, id,
		entity.ProjectJobSchema.Status.Set(StatusSucceeded),
		entity.ProjectJobSchema.Result.Set(strings.TrimSpace(result)),
		entity.ProjectJobSchema.LockedBy.Set(""),
		entity.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		entity.ProjectJobSchema.LastError.Set(""),
		entity.ProjectJobSchema.FinishedAt.Set(now),
		entity.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("mark project job succeeded: %w", err)
	}
	return nil
}

func (r *Repository) ScheduleByID(ctx context.Context, id int64, runAfter time.Time) error {
	if runAfter.IsZero() {
		runAfter = time.Now().UTC()
	} else {
		runAfter = runAfter.UTC()
	}
	if _, err := r.base.UpdateByID(ctx, id,
		entity.ProjectJobSchema.RunAfter.Set(runAfter),
		entity.ProjectJobSchema.UpdatedAt.Set(time.Now().UTC()),
	); err != nil {
		return fmt.Errorf("schedule project job: %w", err)
	}
	return nil
}

func (r *Repository) MarkFailed(ctx context.Context, item entity.ProjectJob, message string, retryAfter time.Duration) error {
	now := time.Now().UTC()
	lastError := strings.TrimSpace(message)
	status := StatusFailed
	finishedAt := now
	runAfter := item.RunAfter
	if item.MaxAttempts <= 0 {
		item.MaxAttempts = 1
	}
	if item.Attempts < item.MaxAttempts {
		status = StatusPending
		finishedAt = time.Time{}
		runAfter = now.Add(retryAfter)
	}
	if _, err := r.base.UpdateByID(ctx, item.ID,
		entity.ProjectJobSchema.Status.Set(status),
		entity.ProjectJobSchema.RunAfter.Set(runAfter),
		entity.ProjectJobSchema.LockedBy.Set(""),
		entity.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		entity.ProjectJobSchema.LastError.Set(lastError),
		entity.ProjectJobSchema.FinishedAt.Set(finishedAt),
		entity.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("mark project job failed: %w", err)
	}
	return nil
}

func (r *Repository) CancelByID(ctx context.Context, id int64) error {
	now := time.Now().UTC()
	if _, err := r.base.UpdateByID(ctx, id,
		entity.ProjectJobSchema.Status.Set(StatusCancelled),
		entity.ProjectJobSchema.LockedBy.Set(""),
		entity.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		entity.ProjectJobSchema.FinishedAt.Set(now),
		entity.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("cancel project job: %w", err)
	}
	return nil
}

func (r *Repository) RetryByID(ctx context.Context, id int64, runAfter time.Time) error {
	now := time.Now().UTC()
	if runAfter.IsZero() {
		runAfter = now
	} else {
		runAfter = runAfter.UTC()
	}
	if _, err := r.base.UpdateByID(ctx, id,
		entity.ProjectJobSchema.Status.Set(StatusPending),
		entity.ProjectJobSchema.Attempts.Set(0),
		entity.ProjectJobSchema.Result.Set(""),
		entity.ProjectJobSchema.RunAfter.Set(runAfter),
		entity.ProjectJobSchema.LockedBy.Set(""),
		entity.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		entity.ProjectJobSchema.StartedAt.Set(time.Time{}),
		entity.ProjectJobSchema.FinishedAt.Set(time.Time{}),
		entity.ProjectJobSchema.LastError.Set(""),
		entity.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("retry project job: %w", err)
	}
	return nil
}
