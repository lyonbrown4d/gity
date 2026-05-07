package projectjobartifact

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/entity"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectJobArtifact, entity.ProjectJobArtifactSchemaDef]
}

type CreateInput struct {
	ProjectID    int64
	ProjectJobID int64
	Name         string
	FileName     string
	FilePath     string
	ContentType  string
}

type StoreInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectJobArtifact](db, entity.ProjectJobArtifactSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[entity.ProjectJobArtifact], error) {
	query := dbx.Select(entity.ProjectJobArtifactSchema.AllColumns().Values()...).
		From(entity.ProjectJobArtifactSchema).
		Where(dbx.And(
			entity.ProjectJobArtifactSchema.ProjectID.Eq(projectID),
			entity.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(entity.ProjectJobArtifactSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectJobAndID(ctx context.Context, projectID int64, projectJobID int64, artifactID int64) (entity.ProjectJobArtifact, error) {
	query := dbx.Select(entity.ProjectJobArtifactSchema.AllColumns().Values()...).
		From(entity.ProjectJobArtifactSchema).
		Where(dbx.And(
			entity.ProjectJobArtifactSchema.ProjectID.Eq(projectID),
			entity.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID),
			entity.ProjectJobArtifactSchema.ID.Eq(artifactID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.ProjectJobArtifact, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectJobArtifact, error) {
	now := time.Now().UTC()
	item := entity.ProjectJobArtifact{
		ProjectID:    input.ProjectID,
		ProjectJobID: input.ProjectJobID,
		Name:         strings.TrimSpace(input.Name),
		FileName:     strings.TrimSpace(input.FileName),
		FilePath:     strings.TrimSpace(input.FilePath),
		ContentType:  strings.TrimSpace(input.ContentType),
		CreatedAt:    now,
		UpdatedAt:    now,
	}
	if item.Name == "" {
		item.Name = item.FileName
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectJobArtifact{}, fmt.Errorf("insert project job artifact: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, id int64, input StoreInput) error {
	if _, err := r.base.UpdateByID(ctx, id,
		entity.ProjectJobArtifactSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		entity.ProjectJobArtifactSchema.ByteSize.Set(input.ByteSize),
		entity.ProjectJobArtifactSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		entity.ProjectJobArtifactSchema.UpdatedAt.Set(time.Now().UTC()),
	); err != nil {
		return fmt.Errorf("mark project job artifact stored: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project job artifact: %w", err)
	}
	return nil
}
