package projectpackageversion

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectPackageVersion, entity.ProjectPackageVersionSchemaDef]
}

type CreateInput struct {
	ProjectPackageID int64
	Version          string
	Status           string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectPackageVersion](db, entity.ProjectPackageVersionSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByPackageID(ctx context.Context, packageID int64) (collectionx.List[entity.ProjectPackageVersion], error) {
	query := dbx.Select(entity.ProjectPackageVersionSchema.AllColumns().Values()...).From(entity.ProjectPackageVersionSchema).Where(entity.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).OrderBy(entity.ProjectPackageVersionSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.ProjectPackageVersion, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) GetByPackageAndVersion(ctx context.Context, packageID int64, version string) (entity.ProjectPackageVersion, error) {
	query := dbx.Select(entity.ProjectPackageVersionSchema.AllColumns().Values()...).From(entity.ProjectPackageVersionSchema).Where(entity.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).Where(entity.ProjectPackageVersionSchema.Version.Eq(strings.TrimSpace(version))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectPackageVersion, error) {
	status := strings.TrimSpace(input.Status)
	if status == "" {
		status = "default"
	}
	now := time.Now().UTC()
	item := entity.ProjectPackageVersion{ProjectPackageID: input.ProjectPackageID, Version: strings.TrimSpace(input.Version), Status: status, CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectPackageVersion{}, fmt.Errorf("insert project package version: %w", err)
	}
	return item, nil
}
