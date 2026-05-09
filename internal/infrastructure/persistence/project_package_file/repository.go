package projectpackagefile

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
	base *dbxrepo.Base[packagedomain.ProjectPackageFile, dbschema.ProjectPackageFileSchemaDef]
}

type CreateInput = packageports.CreateProjectPackageFileInput

type StoreInput = packageports.StoreProjectPackageFileInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[packagedomain.ProjectPackageFile](db, dbschema.ProjectPackageFileSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectPackageFileRepository(repo *Repository) packageports.ProjectPackageFileRepository {
	return repo
}

func (r *Repository) ListByVersionID(ctx context.Context, versionID int64) (*collectionx.List[packagedomain.ProjectPackageFile], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageFileSchema.ProjectPackageVersionID.Eq(versionID)).
		OrderBy(dbschema.ProjectPackageFileSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) ListByVersionIDs(ctx context.Context, versionIDs ...int64) (*collectionx.List[packagedomain.ProjectPackageFile], error) {
	if len(versionIDs) == 0 {
		return collectionx.NewList[packagedomain.ProjectPackageFile](), nil
	}
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPackageFileSchema.ProjectPackageVersionID.In(versionIDs...)).
		OrderBy(dbschema.ProjectPackageFileSchema.ProjectPackageVersionID.Desc(), dbschema.ProjectPackageFileSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageFile, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectPackageFileSchema.ID).Get(ctx, id))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (packagedomain.ProjectPackageFile, error) {
	now := time.Now().UTC()
	item := packagedomain.ProjectPackageFile{ProjectPackageVersionID: input.ProjectPackageVersionID, FileName: strings.TrimSpace(input.FileName), FilePath: strings.TrimSpace(input.FilePath), ContentType: strings.TrimSpace(input.ContentType), StorageKey: fmt.Sprintf("pending/%d", time.Now().UnixNano()), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return packagedomain.ProjectPackageFile{}, fmt.Errorf("insert project package file: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, fileID int64, input StoreInput) error {
	_, err := dbxrepo.PatchSet(r.base, projectPackageFileKey(fileID)).Set(
		dbschema.ProjectPackageFileSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		dbschema.ProjectPackageFileSchema.ByteSize.Set(input.ByteSize),
		dbschema.ProjectPackageFileSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		dbschema.ProjectPackageFileSchema.UpdatedAt.Set(time.Now().UTC()),
	).Apply(ctx)
	if err != nil {
		return fmt.Errorf("update project package file: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, fileID int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectPackageFileKey(fileID)); err != nil {
		return fmt.Errorf("delete project package file: %w", err)
	}
	return nil
}

func projectPackageFileKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectPackageFileSchema.ID, id))
}
