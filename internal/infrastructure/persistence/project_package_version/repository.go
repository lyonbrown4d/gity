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
	"github.com/arcgolabs/dbx/querydsl"
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
	query := querydsl.Select(dbschema.ProjectPackageVersionSchema.AllColumns().Values()...).From(dbschema.ProjectPackageVersionSchema).Where(dbschema.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).OrderBy(dbschema.ProjectPackageVersionSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageVersion, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectPackageVersionSchema.ID).Get(ctx, id))
}

func (r *Repository) GetByPackageAndVersion(ctx context.Context, packageID int64, version string) (packagedomain.ProjectPackageVersion, error) {
	query := querydsl.Select(dbschema.ProjectPackageVersionSchema.AllColumns().Values()...).From(dbschema.ProjectPackageVersionSchema).Where(querydsl.And(dbschema.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID), dbschema.ProjectPackageVersionSchema.Version.Eq(strings.TrimSpace(version)))).Limit(1)
	return persistence.One(r.base.First(ctx, query))
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
