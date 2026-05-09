package projectpackageversion

import (
	"context"
	"fmt"
	packageports "github.com/DaiYuANg/gity/internal/application/ports"
	packagedomain "github.com/DaiYuANg/gity/internal/domain/package_registry"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[packagedomain.ProjectPackageVersion, dbschema.ProjectPackageVersionSchemaDef]
}

type CreateInput = packageports.CreateProjectPackageVersionInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[packagedomain.ProjectPackageVersion](db, dbschema.ProjectPackageVersionSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectPackageVersionRepository(repo *Repository) packageports.ProjectPackageVersionRepository {
	return repo
}

func (r *Repository) ListByPackageID(ctx context.Context, packageID int64) (*collectionx.List[packagedomain.ProjectPackageVersion], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).
		OrderBy(dbschema.ProjectPackageVersionSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageVersion, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectPackageVersionSchema.ID).Get(ctx, id))
}

func (r *Repository) GetByPackageAndVersion(ctx context.Context, packageID int64, version string) (packagedomain.ProjectPackageVersion, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).
		Where(dbschema.ProjectPackageVersionSchema.Version.Eq(strings.TrimSpace(version))).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (packagedomain.ProjectPackageVersion, error) {
	status := strings.TrimSpace(input.Status)
	if status == "" {
		status = "default"
	}
	now := time.Now().UTC()
	item := packagedomain.ProjectPackageVersion{ProjectPackageID: input.ProjectPackageID, Version: strings.TrimSpace(input.Version), Status: status, CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return packagedomain.ProjectPackageVersion{}, fmt.Errorf("insert project package version: %w", err)
	}
	return item, nil
}
