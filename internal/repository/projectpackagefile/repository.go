package projectpackagefile

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectPackageFile, entity.ProjectPackageFileSchemaDef]
}

type CreateInput struct {
	ProjectPackageVersionID int64
	FileName                string
	FilePath                string
	ContentType             string
}

type StoreInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectPackageFile](db, entity.ProjectPackageFileSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByVersionID(ctx context.Context, versionID int64) (*collectionx.List[entity.ProjectPackageFile], error) {
	query := querydsl.Select(entity.ProjectPackageFileSchema.AllColumns().Values()...).From(entity.ProjectPackageFileSchema).Where(entity.ProjectPackageFileSchema.ProjectPackageVersionID.Eq(versionID)).OrderBy(entity.ProjectPackageFileSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) ListByVersionIDs(ctx context.Context, versionIDs ...int64) (*collectionx.List[entity.ProjectPackageFile], error) {
	if len(versionIDs) == 0 {
		return collectionx.NewList[entity.ProjectPackageFile](), nil
	}
	query := querydsl.Select(entity.ProjectPackageFileSchema.AllColumns().Values()...).From(entity.ProjectPackageFileSchema).Where(entity.ProjectPackageFileSchema.ProjectPackageVersionID.In(versionIDs...)).OrderBy(entity.ProjectPackageFileSchema.ProjectPackageVersionID.Desc(), entity.ProjectPackageFileSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.ProjectPackageFile, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectPackageFile, error) {
	now := time.Now().UTC()
	item := entity.ProjectPackageFile{ProjectPackageVersionID: input.ProjectPackageVersionID, FileName: strings.TrimSpace(input.FileName), FilePath: strings.TrimSpace(input.FilePath), ContentType: strings.TrimSpace(input.ContentType), StorageKey: fmt.Sprintf("pending/%d", time.Now().UnixNano()), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectPackageFile{}, fmt.Errorf("insert project package file: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, fileID int64, input StoreInput) error {
	_, err := r.base.UpdateByID(ctx, fileID,
		entity.ProjectPackageFileSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		entity.ProjectPackageFileSchema.ByteSize.Set(input.ByteSize),
		entity.ProjectPackageFileSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		entity.ProjectPackageFileSchema.UpdatedAt.Set(time.Now().UTC()),
	)
	if err != nil {
		return fmt.Errorf("update project package file: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, fileID int64) error {
	if _, err := r.base.DeleteByID(ctx, fileID); err != nil {
		return fmt.Errorf("delete project package file: %w", err)
	}
	return nil
}
