package projectpackage

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
	base *dbxrepo.Base[entity.ProjectPackage, entity.ProjectPackageSchemaDef]
}

type CreateInput struct {
	ProjectID int64
	Type      string
	Name      string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectPackage](db, entity.ProjectPackageSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (collectionx.List[entity.ProjectPackage], error) {
	query := dbx.Select(entity.ProjectPackageSchema.AllColumns().Values()...).From(entity.ProjectPackageSchema).Where(entity.ProjectPackageSchema.ProjectID.Eq(projectID)).OrderBy(entity.ProjectPackageSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.ProjectPackage, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) GetByProjectTypeAndName(ctx context.Context, projectID int64, packageType string, name string) (entity.ProjectPackage, error) {
	query := dbx.Select(entity.ProjectPackageSchema.AllColumns().Values()...).From(entity.ProjectPackageSchema).Where(entity.ProjectPackageSchema.ProjectID.Eq(projectID)).Where(entity.ProjectPackageSchema.Type.Eq(strings.TrimSpace(packageType))).Where(entity.ProjectPackageSchema.Name.Eq(strings.TrimSpace(name))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectPackage, error) {
	now := time.Now().UTC()
	item := entity.ProjectPackage{ProjectID: input.ProjectID, Type: strings.TrimSpace(input.Type), Name: strings.TrimSpace(input.Name), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectPackage{}, fmt.Errorf("insert project package: %w", err)
	}
	return item, nil
}
