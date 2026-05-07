package projectpackageversion

import (
	"context"
	"fmt"
	packagedomain "github.com/DaiYuANg/gity/internal/domain/packageregistry"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[packagedomain.ProjectPackageVersion, packagedomain.ProjectPackageVersionSchemaDef]
}

type CreateInput struct {
	ProjectPackageID int64
	Version          string
	Status           string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[packagedomain.ProjectPackageVersion](db, packagedomain.ProjectPackageVersionSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByPackageID(ctx context.Context, packageID int64) (*collectionx.List[packagedomain.ProjectPackageVersion], error) {
	query := querydsl.Select(packagedomain.ProjectPackageVersionSchema.AllColumns().Values()...).From(packagedomain.ProjectPackageVersionSchema).Where(packagedomain.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID)).OrderBy(packagedomain.ProjectPackageVersionSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageVersion, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) GetByPackageAndVersion(ctx context.Context, packageID int64, version string) (packagedomain.ProjectPackageVersion, error) {
	query := querydsl.Select(packagedomain.ProjectPackageVersionSchema.AllColumns().Values()...).From(packagedomain.ProjectPackageVersionSchema).Where(querydsl.And(packagedomain.ProjectPackageVersionSchema.ProjectPackageID.Eq(packageID), packagedomain.ProjectPackageVersionSchema.Version.Eq(strings.TrimSpace(version)))).Limit(1)
	return r.base.First(ctx, query)
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
