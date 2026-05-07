package projectjob

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

const (
	StatusPending   = "pending"
	StatusRunning   = "running"
	StatusSucceeded = "succeeded"
	StatusFailed    = "failed"
	StatusCancelled = "cancelled"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJob, cidomain.ProjectJobSchemaDef]
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
		base: dbxrepo.NewWithOptions[cidomain.ProjectJob](db, cidomain.ProjectJobSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectJob], error) {
	query := querydsl.Select(cidomain.ProjectJobSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobSchema).
		Where(cidomain.ProjectJobSchema.ProjectID.Eq(projectID)).
		OrderBy(cidomain.ProjectJobSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectJob, error) {
	query := querydsl.Select(cidomain.ProjectJobSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobSchema).
		Where(querydsl.And(
			cidomain.ProjectJobSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobSchema.ID.Eq(id),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (cidomain.ProjectJob, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectJob, error) {
	now := time.Now().UTC()
	runAfter := input.RunAfter.UTC()
	if runAfter.IsZero() {
		runAfter = now
	}
	maxAttempts := input.MaxAttempts
	if maxAttempts <= 0 {
		maxAttempts = 3
	}
	item := cidomain.ProjectJob{
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
		return cidomain.ProjectJob{}, fmt.Errorf("insert project job: %w", err)
	}
	return item, nil
}

func (r *Repository) ClaimNext(ctx context.Context, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	return r.claimNext(ctx, 0, nil, workerID, lease)
}

func (r *Repository) ClaimNextByKinds(ctx context.Context, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	return r.claimNext(ctx, 0, kinds, workerID, lease)
}

func (r *Repository) ClaimNextByProjectID(ctx context.Context, projectID int64, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	return r.claimNext(ctx, projectID, nil, workerID, lease)
}

func (r *Repository) ClaimNextByProjectIDAndKinds(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	return r.claimNext(ctx, projectID, kinds, workerID, lease)
}

func (r *Repository) claimNext(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	if lease <= 0 {
		lease = time.Minute
	}
	now := time.Now().UTC()
	predicates := []querydsl.Predicate{
		cidomain.ProjectJobSchema.Status.Eq(StatusPending),
		cidomain.ProjectJobSchema.RunAfter.Le(now),
	}
	if projectID > 0 {
		predicates = append(predicates, cidomain.ProjectJobSchema.ProjectID.Eq(projectID))
	}
	if len(kinds) > 0 {
		predicates = append(predicates, cidomain.ProjectJobSchema.Kind.In(kinds...))
	}
	query := querydsl.Select(cidomain.ProjectJobSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobSchema).
		Where(querydsl.And(predicates...)).
		OrderBy(cidomain.ProjectJobSchema.RunAfter.Asc(), cidomain.ProjectJobSchema.ID.Asc()).
		Limit(1)
	item, err := r.base.First(ctx, query)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return cidomain.ProjectJob{}, false, nil
		}
		return cidomain.ProjectJob{}, false, err
	}
	item.Status = StatusRunning
	item.Attempts++
	item.LockedBy = strings.TrimSpace(workerID)
	item.LockedUntil = now.Add(lease)
	item.StartedAt = now
	item.UpdatedAt = now
	if _, err := r.base.UpdateByID(ctx, item.ID,
		cidomain.ProjectJobSchema.Status.Set(item.Status),
		cidomain.ProjectJobSchema.Attempts.Set(item.Attempts),
		cidomain.ProjectJobSchema.LockedBy.Set(item.LockedBy),
		cidomain.ProjectJobSchema.LockedUntil.Set(item.LockedUntil),
		cidomain.ProjectJobSchema.StartedAt.Set(item.StartedAt),
		cidomain.ProjectJobSchema.UpdatedAt.Set(item.UpdatedAt),
	); err != nil {
		return cidomain.ProjectJob{}, false, fmt.Errorf("claim project job: %w", err)
	}
	return item, true, nil
}

func (r *Repository) MarkSucceeded(ctx context.Context, id int64, result string) error {
	now := time.Now().UTC()
	if _, err := r.base.UpdateByID(ctx, id,
		cidomain.ProjectJobSchema.Status.Set(StatusSucceeded),
		cidomain.ProjectJobSchema.Result.Set(strings.TrimSpace(result)),
		cidomain.ProjectJobSchema.LockedBy.Set(""),
		cidomain.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		cidomain.ProjectJobSchema.LastError.Set(""),
		cidomain.ProjectJobSchema.FinishedAt.Set(now),
		cidomain.ProjectJobSchema.UpdatedAt.Set(now),
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
		cidomain.ProjectJobSchema.RunAfter.Set(runAfter),
		cidomain.ProjectJobSchema.UpdatedAt.Set(time.Now().UTC()),
	); err != nil {
		return fmt.Errorf("schedule project job: %w", err)
	}
	return nil
}

func (r *Repository) MarkFailed(ctx context.Context, item cidomain.ProjectJob, message string, retryAfter time.Duration) error {
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
		cidomain.ProjectJobSchema.Status.Set(status),
		cidomain.ProjectJobSchema.RunAfter.Set(runAfter),
		cidomain.ProjectJobSchema.LockedBy.Set(""),
		cidomain.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		cidomain.ProjectJobSchema.LastError.Set(lastError),
		cidomain.ProjectJobSchema.FinishedAt.Set(finishedAt),
		cidomain.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("mark project job failed: %w", err)
	}
	return nil
}

func (r *Repository) CancelByID(ctx context.Context, id int64) error {
	now := time.Now().UTC()
	if _, err := r.base.UpdateByID(ctx, id,
		cidomain.ProjectJobSchema.Status.Set(StatusCancelled),
		cidomain.ProjectJobSchema.LockedBy.Set(""),
		cidomain.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		cidomain.ProjectJobSchema.FinishedAt.Set(now),
		cidomain.ProjectJobSchema.UpdatedAt.Set(now),
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
		cidomain.ProjectJobSchema.Status.Set(StatusPending),
		cidomain.ProjectJobSchema.Attempts.Set(0),
		cidomain.ProjectJobSchema.Result.Set(""),
		cidomain.ProjectJobSchema.RunAfter.Set(runAfter),
		cidomain.ProjectJobSchema.LockedBy.Set(""),
		cidomain.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		cidomain.ProjectJobSchema.StartedAt.Set(time.Time{}),
		cidomain.ProjectJobSchema.FinishedAt.Set(time.Time{}),
		cidomain.ProjectJobSchema.LastError.Set(""),
		cidomain.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("retry project job: %w", err)
	}
	return nil
}
