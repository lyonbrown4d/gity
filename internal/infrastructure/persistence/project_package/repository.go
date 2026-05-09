package projectpackage

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
	base *dbxrepo.Base[packagedomain.ProjectPackage, dbschema.ProjectPackageSchemaDef]
}

type CreateInput = packageports.CreateProjectPackageInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[packagedomain.ProjectPackage](db, dbschema.ProjectPackageSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectPackageRepository(repo *Repository) packageports.ProjectPackageRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[packagedomain.ProjectPackage], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectPackageSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackage, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectPackageSchema.ID).Get(ctx, id))
}

func (r *Repository) GetByProjectTypeAndName(ctx context.Context, projectID int64, packageType, name string) (packagedomain.ProjectPackage, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectPackageSchema.Type.Eq(strings.TrimSpace(packageType))).
		Where(dbschema.ProjectPackageSchema.Name.Eq(strings.TrimSpace(name))).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (packagedomain.ProjectPackage, error) {
	now := time.Now().UTC()
	item := packagedomain.ProjectPackage{ProjectID: input.ProjectID, Type: strings.TrimSpace(input.Type), Name: strings.TrimSpace(input.Name), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return packagedomain.ProjectPackage{}, fmt.Errorf("insert project package: %w", err)
	}
	return item, nil
}
