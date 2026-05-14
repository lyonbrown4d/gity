package projectjob

import (
	"context"
	"fmt"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	ciports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
	"strings"
	"time"
)

const (
	StatusPending   = ciports.ProjectJobStatusPending
	StatusRunning   = ciports.ProjectJobStatusRunning
	StatusSucceeded = ciports.ProjectJobStatusSucceeded
	StatusFailed    = ciports.ProjectJobStatusFailed
	StatusCancelled = ciports.ProjectJobStatusCancelled
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJob, dbschema.ProjectJobSchemaDef]
}

type CreateInput = ciports.CreateProjectJobInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectJob](db, dbschema.ProjectJobSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectJobRepository(repo *Repository) ciports.ProjectJobRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectJob], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectJobSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (cidomain.ProjectJob, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobSchema.ID.Eq(id)).
		First(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (cidomain.ProjectJob, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectJobSchema.ID).Get(ctx, id))
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

func (r *Repository) RequeueExpiredLeases(ctx context.Context, now time.Time) (int64, error) {
	if now.IsZero() {
		now = time.Now().UTC()
	} else {
		now = now.UTC()
	}
	items, err := dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobSchema.Status.Eq(StatusRunning)).
		Where(dbschema.ProjectJobSchema.LockedUntil.Le(now)).
		OrderBy(dbschema.ProjectJobSchema.LockedUntil.Asc(), dbschema.ProjectJobSchema.ID.Asc()).
		List(ctx)
	if err != nil {
		return 0, oops.In("persistence.project_job").Wrapf(err, "list expired project job leases")
	}
	var expired int64
	values := items.Values()
	for index := range values {
		item := values[index]
		if err := r.MarkFailed(ctx, item, "runner lease expired", 0); err != nil {
			return expired, oops.In("persistence.project_job").With("project_id", item.ProjectID, "job_id", item.ID).Wrapf(err, "requeue expired project job lease")
		}
		expired++
	}
	return expired, nil
}

func (r *Repository) claimNext(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	if lease <= 0 {
		lease = time.Minute
	}
	now := time.Now().UTC()
	predicates := []querydsl.Predicate{
		dbschema.ProjectJobSchema.Status.Eq(StatusPending),
		dbschema.ProjectJobSchema.RunAfter.Le(now),
	}
	if projectID > 0 {
		predicates = append(predicates, dbschema.ProjectJobSchema.ProjectID.Eq(projectID))
	}
	if len(kinds) > 0 {
		predicates = append(predicates, dbschema.ProjectJobSchema.Kind.In(kinds...))
	}
	item, err := dbxrepo.Query(r.base).
		Where(querydsl.And(predicates...)).
		OrderBy(dbschema.ProjectJobSchema.RunAfter.Asc(), dbschema.ProjectJobSchema.ID.Asc()).
		First(ctx)
	if err != nil {
		if persistence.IsNotFound(err) {
			return cidomain.ProjectJob{}, false, nil
		}
		return cidomain.ProjectJob{}, false, oops.In("persistence.project_job").With("project_id", projectID, "worker_id", workerID).Wrapf(err, "claim next project job")
	}
	item.Status = StatusRunning
	item.Attempts++
	item.LockedBy = strings.TrimSpace(workerID)
	item.LockedUntil = now.Add(lease)
	item.StartedAt = now
	item.UpdatedAt = now
	if err := r.patchByID(ctx, item.ID,
		dbschema.ProjectJobSchema.Status.Set(item.Status),
		dbschema.ProjectJobSchema.Attempts.Set(item.Attempts),
		dbschema.ProjectJobSchema.LockedBy.Set(item.LockedBy),
		dbschema.ProjectJobSchema.LockedUntil.Set(item.LockedUntil),
		dbschema.ProjectJobSchema.StartedAt.Set(item.StartedAt),
		dbschema.ProjectJobSchema.UpdatedAt.Set(item.UpdatedAt),
	); err != nil {
		return cidomain.ProjectJob{}, false, fmt.Errorf("claim project job: %w", err)
	}
	return item, true, nil
}

func (r *Repository) MarkSucceeded(ctx context.Context, id int64, result string) error {
	now := time.Now().UTC()
	if err := r.patchByID(ctx, id,
		dbschema.ProjectJobSchema.Status.Set(StatusSucceeded),
		dbschema.ProjectJobSchema.Result.Set(strings.TrimSpace(result)),
		dbschema.ProjectJobSchema.LockedBy.Set(""),
		dbschema.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		dbschema.ProjectJobSchema.LastError.Set(""),
		dbschema.ProjectJobSchema.FinishedAt.Set(now),
		dbschema.ProjectJobSchema.UpdatedAt.Set(now),
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
	if err := r.patchByID(ctx, id,
		dbschema.ProjectJobSchema.RunAfter.Set(runAfter),
		dbschema.ProjectJobSchema.UpdatedAt.Set(time.Now().UTC()),
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
	if err := r.patchByID(ctx, item.ID,
		dbschema.ProjectJobSchema.Status.Set(status),
		dbschema.ProjectJobSchema.RunAfter.Set(runAfter),
		dbschema.ProjectJobSchema.LockedBy.Set(""),
		dbschema.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		dbschema.ProjectJobSchema.LastError.Set(lastError),
		dbschema.ProjectJobSchema.FinishedAt.Set(finishedAt),
		dbschema.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("mark project job failed: %w", err)
	}
	return nil
}

func (r *Repository) CancelByID(ctx context.Context, id int64) error {
	now := time.Now().UTC()
	if err := r.patchByID(ctx, id,
		dbschema.ProjectJobSchema.Status.Set(StatusCancelled),
		dbschema.ProjectJobSchema.LockedBy.Set(""),
		dbschema.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		dbschema.ProjectJobSchema.FinishedAt.Set(now),
		dbschema.ProjectJobSchema.UpdatedAt.Set(now),
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
	if err := r.patchByID(ctx, id,
		dbschema.ProjectJobSchema.Status.Set(StatusPending),
		dbschema.ProjectJobSchema.Attempts.Set(0),
		dbschema.ProjectJobSchema.Result.Set(""),
		dbschema.ProjectJobSchema.RunAfter.Set(runAfter),
		dbschema.ProjectJobSchema.LockedBy.Set(""),
		dbschema.ProjectJobSchema.LockedUntil.Set(time.Time{}),
		dbschema.ProjectJobSchema.StartedAt.Set(time.Time{}),
		dbschema.ProjectJobSchema.FinishedAt.Set(time.Time{}),
		dbschema.ProjectJobSchema.LastError.Set(""),
		dbschema.ProjectJobSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("retry project job: %w", err)
	}
	return nil
}

func (r *Repository) patchByID(ctx context.Context, id int64, assignments ...querydsl.Assignment) error {
	_, err := dbxrepo.PatchSet(r.base, projectJobKey(id)).Set(assignments...).Apply(ctx)
	if err != nil {
		return oops.In("persistence.project_job").With("id", id).Wrapf(err, "patch project job")
	}
	return nil
}

func projectJobKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectJobSchema.ID, id))
}
