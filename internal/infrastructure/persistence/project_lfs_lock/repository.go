package projectlfslock

import (
	"context"
	"fmt"
	lfsports "github.com/DaiYuANg/gity/internal/application/ports"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
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
	base *dbxrepo.Base[lfsdomain.ProjectLFSLock, dbschema.ProjectLFSLockSchemaDef]
}

type CreateInput = lfsports.CreateProjectLFSLockInput

type ListInput = lfsports.ListProjectLFSLocksInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[lfsdomain.ProjectLFSLock](db, dbschema.ProjectLFSLockSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectLFSLockRepository(repo *Repository) lfsports.ProjectLFSLockRepository {
	return repo
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (lfsdomain.ProjectLFSLock, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectLFSLockSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectLFSLockSchema.ID.Eq(id)).
		First(ctx))
}

func (r *Repository) GetByProjectAndPath(ctx context.Context, projectID int64, path string) (lfsdomain.ProjectLFSLock, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectLFSLockSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(path))).
		First(ctx))
}

func (r *Repository) ListByProjectID(ctx context.Context, input ListInput) (*collectionx.List[lfsdomain.ProjectLFSLock], error) {
	limit := input.Limit
	if limit <= 0 {
		limit = 100
	}
	predicates := []querydsl.Predicate{dbschema.ProjectLFSLockSchema.ProjectID.Eq(input.ProjectID)}
	if strings.TrimSpace(input.Path) != "" {
		predicates = append(predicates, dbschema.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(input.Path)))
	}
	if input.AfterID > 0 {
		predicates = append(predicates, dbschema.ProjectLFSLockSchema.ID.Gt(input.AfterID))
	}
	return persistence.Many(dbxrepo.Query(r.base).
		Where(querydsl.And(predicates...)).
		OrderBy(dbschema.ProjectLFSLockSchema.ID.Asc()).
		Limit(limit).
		List(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (lfsdomain.ProjectLFSLock, error) {
	now := time.Now().UTC()
	item := lfsdomain.ProjectLFSLock{ProjectID: input.ProjectID, OwnerUserID: input.OwnerUserID, Path: strings.TrimSpace(input.Path), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return lfsdomain.ProjectLFSLock{}, fmt.Errorf("insert project lfs lock: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectLFSLockKey(id)); err != nil {
		return fmt.Errorf("delete project lfs lock: %w", err)
	}
	return nil
}

func projectLFSLockKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectLFSLockSchema.ID, id))
}
