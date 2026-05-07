package projectlfslock

import (
	"context"
	"fmt"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[lfsdomain.ProjectLFSLock, lfsdomain.ProjectLFSLockSchemaDef]
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
	return &Repository{base: dbxrepo.NewWithOptions[lfsdomain.ProjectLFSLock](db, lfsdomain.ProjectLFSLockSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (lfsdomain.ProjectLFSLock, error) {
	query := querydsl.Select(lfsdomain.ProjectLFSLockSchema.AllColumns().Values()...).From(lfsdomain.ProjectLFSLockSchema).Where(querydsl.And(lfsdomain.ProjectLFSLockSchema.ProjectID.Eq(projectID), lfsdomain.ProjectLFSLockSchema.ID.Eq(id))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectAndPath(ctx context.Context, projectID int64, path string) (lfsdomain.ProjectLFSLock, error) {
	query := querydsl.Select(lfsdomain.ProjectLFSLockSchema.AllColumns().Values()...).From(lfsdomain.ProjectLFSLockSchema).Where(querydsl.And(lfsdomain.ProjectLFSLockSchema.ProjectID.Eq(projectID), lfsdomain.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(path)))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) ListByProjectID(ctx context.Context, input ListInput) (*collectionx.List[lfsdomain.ProjectLFSLock], error) {
	limit := input.Limit
	if limit <= 0 {
		limit = 100
	}
	predicates := []querydsl.Predicate{lfsdomain.ProjectLFSLockSchema.ProjectID.Eq(input.ProjectID)}
	if strings.TrimSpace(input.Path) != "" {
		predicates = append(predicates, lfsdomain.ProjectLFSLockSchema.Path.Eq(strings.TrimSpace(input.Path)))
	}
	if input.AfterID > 0 {
		predicates = append(predicates, lfsdomain.ProjectLFSLockSchema.ID.Gt(input.AfterID))
	}
	query := querydsl.Select(lfsdomain.ProjectLFSLockSchema.AllColumns().Values()...).From(lfsdomain.ProjectLFSLockSchema).Where(querydsl.And(predicates...)).OrderBy(lfsdomain.ProjectLFSLockSchema.ID.Asc()).Limit(limit)
	return r.base.List(ctx, query)
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
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project lfs lock: %w", err)
	}
	return nil
}
