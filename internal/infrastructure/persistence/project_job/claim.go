package projectjob

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

func (r *Repository) ListClaimableByProjectIDAndKinds(ctx context.Context, projectID int64, kinds []string, limit int) (*collectionx.List[cidomain.ProjectJob], error) {
	if limit <= 0 {
		limit = 50
	}
	now := time.Now().UTC()
	predicates := []querydsl.Predicate{
		dbschema.ProjectJobSchema.ProjectID.Eq(projectID),
		dbschema.ProjectJobSchema.Status.Eq(StatusPending),
		dbschema.ProjectJobSchema.RunAfter.Le(now),
	}
	if len(kinds) > 0 {
		predicates = append(predicates, dbschema.ProjectJobSchema.Kind.In(kinds...))
	}
	items, err := dbxrepo.Query(r.base).
		Where(querydsl.And(predicates...)).
		OrderBy(dbschema.ProjectJobSchema.RunAfter.Asc(), dbschema.ProjectJobSchema.ID.Asc()).
		Limit(limit).
		List(ctx)
	return persistence.Many(items, err)
}

func (r *Repository) ClaimByID(ctx context.Context, id int64, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	if lease <= 0 {
		lease = time.Minute
	}
	now := time.Now().UTC()
	item, err := r.GetByID(ctx, id)
	if err != nil {
		return cidomain.ProjectJob{}, false, err
	}
	if item.Status != StatusPending || item.RunAfter.After(now) {
		return cidomain.ProjectJob{}, false, nil
	}
	item = claimedProjectJob(item, workerID, lease, now)
	if err := r.patchClaimed(ctx, item); err != nil {
		return cidomain.ProjectJob{}, false, fmt.Errorf("claim project job by id: %w", err)
	}
	return item, true, nil
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
	item, found, err := r.nextClaimable(ctx, projectID, kinds, now)
	if err != nil {
		return cidomain.ProjectJob{}, false, err
	}
	if !found {
		return cidomain.ProjectJob{}, false, nil
	}
	item = claimedProjectJob(item, workerID, lease, now)
	if err := r.patchClaimed(ctx, item); err != nil {
		return cidomain.ProjectJob{}, false, fmt.Errorf("claim project job: %w", err)
	}
	return item, true, nil
}

func (r *Repository) nextClaimable(ctx context.Context, projectID int64, kinds []string, now time.Time) (cidomain.ProjectJob, bool, error) {
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
		return cidomain.ProjectJob{}, false, oops.In("persistence.project_job").With("project_id", projectID).Wrapf(err, "claim next project job")
	}
	return item, true, nil
}

func claimedProjectJob(item cidomain.ProjectJob, workerID string, lease time.Duration, now time.Time) cidomain.ProjectJob {
	item.Status = StatusRunning
	item.Attempts++
	item.LockedBy = strings.TrimSpace(workerID)
	item.LockedUntil = now.Add(lease)
	item.StartedAt = now
	item.UpdatedAt = now
	return item
}

func (r *Repository) patchClaimed(ctx context.Context, item cidomain.ProjectJob) error {
	return r.patchByID(ctx, item.ID,
		dbschema.ProjectJobSchema.Status.Set(item.Status),
		dbschema.ProjectJobSchema.Attempts.Set(item.Attempts),
		dbschema.ProjectJobSchema.LockedBy.Set(item.LockedBy),
		dbschema.ProjectJobSchema.LockedUntil.Set(item.LockedUntil),
		dbschema.ProjectJobSchema.StartedAt.Set(item.StartedAt),
		dbschema.ProjectJobSchema.UpdatedAt.Set(item.UpdatedAt),
	)
}
