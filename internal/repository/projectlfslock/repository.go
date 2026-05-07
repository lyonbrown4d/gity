package projectlfslock

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
	base *dbxrepo.Base[entity.ProjectLFSLock, entity.ProjectLFSLockSchemaDef]
}

type CreateInput struct {
	ProjectID   int64
	OwnerUserID int64
	Path        string
}

type ListInput struct {
	ProjectID int64
	Path      string
	AfterID   int64
	Limit     int
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectLFSLock](db, entity.ProjectLFSLockSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (entity.ProjectLFSLock, error) {
	query := dbx.Select(entity.ProjectLFSLockSchema.AllColumns().Values()...).From(entity.ProjectLFSLockSchema).Where(dbx.And(entity.ProjectLFSLockSchema.ProjectID.Eq(projectID), entity.ProjectLFSLockSchema.ID.Eq(id))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectAndPath(ctx context.Context, projectID int64, path string) (entity.ProjectLFSLock, error) {
	query := dbx.Select(entity.ProjectLFSLockSchema.AllColumns().Values()...).From(entity.ProjectLFSLockSchema).Where(dbx.And(entity.ProjectLFSLockSchema.ProjectID.Eq(projectID), entity.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(path)))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) ListByProjectID(ctx context.Context, input ListInput) (*collectionx.List[entity.ProjectLFSLock], error) {
	limit := input.Limit
	if limit <= 0 {
		limit = 100
	}
	predicates := []dbx.Predicate{entity.ProjectLFSLockSchema.ProjectID.Eq(input.ProjectID)}
	if strings.TrimSpace(input.Path) != "" {
		predicates = append(predicates, entity.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(input.Path)))
	}
	if input.AfterID > 0 {
		predicates = append(predicates, entity.ProjectLFSLockSchema.ID.Gt(input.AfterID))
	}
	query := dbx.Select(entity.ProjectLFSLockSchema.AllColumns().Values()...).From(entity.ProjectLFSLockSchema).Where(dbx.And(predicates...)).OrderBy(entity.ProjectLFSLockSchema.ID.Asc()).Limit(limit)
	return r.base.List(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectLFSLock, error) {
	now := time.Now().UTC()
	item := entity.ProjectLFSLock{ProjectID: input.ProjectID, OwnerUserID: input.OwnerUserID, Path: strings.TrimSpace(input.Path), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectLFSLock{}, fmt.Errorf("insert project lfs lock: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project lfs lock: %w", err)
	}
	return nil
}
