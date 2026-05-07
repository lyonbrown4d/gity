package projectpackage

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
	base *dbxrepo.Base[packagedomain.ProjectPackage, packagedomain.ProjectPackageSchemaDef]
}

type CreateInput struct {
	ProjectID int64
	Type      string
	Name      string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[packagedomain.ProjectPackage](db, packagedomain.ProjectPackageSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[packagedomain.ProjectPackage], error) {
	query := querydsl.Select(packagedomain.ProjectPackageSchema.AllColumns().Values()...).From(packagedomain.ProjectPackageSchema).Where(packagedomain.ProjectPackageSchema.ProjectID.Eq(projectID)).OrderBy(packagedomain.ProjectPackageSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackage, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) GetByProjectTypeAndName(ctx context.Context, projectID int64, packageType string, name string) (packagedomain.ProjectPackage, error) {
	query := querydsl.Select(packagedomain.ProjectPackageSchema.AllColumns().Values()...).From(packagedomain.ProjectPackageSchema).Where(querydsl.And(packagedomain.ProjectPackageSchema.ProjectID.Eq(projectID), packagedomain.ProjectPackageSchema.Type.Eq(strings.TrimSpace(packageType)), packagedomain.ProjectPackageSchema.Name.Eq(strings.TrimSpace(name)))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (packagedomain.ProjectPackage, error) {
	now := time.Now().UTC()
	item := packagedomain.ProjectPackage{ProjectID: input.ProjectID, Type: strings.TrimSpace(input.Type), Name: strings.TrimSpace(input.Name), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return packagedomain.ProjectPackage{}, fmt.Errorf("insert project package: %w", err)
	}
	return item, nil
}
