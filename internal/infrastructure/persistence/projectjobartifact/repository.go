package projectjobartifact

import (
	"context"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJobArtifact, cidomain.ProjectJobArtifactSchemaDef]
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
		base: dbxrepo.NewWithOptions[cidomain.ProjectJobArtifact](db, cidomain.ProjectJobArtifactSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[cidomain.ProjectJobArtifact], error) {
	query := querydsl.Select(cidomain.ProjectJobArtifactSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobArtifactSchema).
		Where(querydsl.And(
			cidomain.ProjectJobArtifactSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID),
		)).
		OrderBy(cidomain.ProjectJobArtifactSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectJobAndID(ctx context.Context, projectID int64, projectJobID int64, artifactID int64) (cidomain.ProjectJobArtifact, error) {
	query := querydsl.Select(cidomain.ProjectJobArtifactSchema.AllColumns().Values()...).
		From(cidomain.ProjectJobArtifactSchema).
		Where(querydsl.And(
			cidomain.ProjectJobArtifactSchema.ProjectID.Eq(projectID),
			cidomain.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID),
			cidomain.ProjectJobArtifactSchema.ID.Eq(artifactID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (cidomain.ProjectJobArtifact, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectJobArtifact, error) {
	now := time.Now().UTC()
	item := cidomain.ProjectJobArtifact{
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
		return cidomain.ProjectJobArtifact{}, fmt.Errorf("insert project job artifact: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, id int64, input StoreInput) error {
	if _, err := r.base.UpdateByID(ctx, id,
		cidomain.ProjectJobArtifactSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		cidomain.ProjectJobArtifactSchema.ByteSize.Set(input.ByteSize),
		cidomain.ProjectJobArtifactSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		cidomain.ProjectJobArtifactSchema.UpdatedAt.Set(time.Now().UTC()),
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
