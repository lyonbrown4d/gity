package projectjobartifact

import (
	"context"
	"fmt"
	ciports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectJobArtifact, dbschema.ProjectJobArtifactSchemaDef]
}

type CreateInput = ciports.CreateProjectJobArtifactInput

type StoreInput = ciports.StoreProjectJobArtifactInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectJobArtifact](db, dbschema.ProjectJobArtifactSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectJobArtifactRepository(repo *Repository) ciports.ProjectJobArtifactRepository {
	return repo
}

func (r *Repository) ListByProjectJobID(ctx context.Context, projectID, projectJobID int64) (*collectionx.List[cidomain.ProjectJobArtifact], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobArtifactSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID)).
		OrderBy(dbschema.ProjectJobArtifactSchema.ID.Asc()).
		List(ctx))
}

func (r *Repository) GetByProjectJobAndID(ctx context.Context, projectID, projectJobID, artifactID int64) (cidomain.ProjectJobArtifact, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectJobArtifactSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectJobArtifactSchema.ProjectJobID.Eq(projectJobID)).
		Where(dbschema.ProjectJobArtifactSchema.ID.Eq(artifactID)).
		First(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (cidomain.ProjectJobArtifact, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectJobArtifactSchema.ID).Get(ctx, id))
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
	if _, err := dbxrepo.PatchSet(r.base, projectJobArtifactKey(id)).Set(
		dbschema.ProjectJobArtifactSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		dbschema.ProjectJobArtifactSchema.ByteSize.Set(input.ByteSize),
		dbschema.ProjectJobArtifactSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		dbschema.ProjectJobArtifactSchema.UpdatedAt.Set(time.Now().UTC()),
	).Apply(ctx); err != nil {
		return fmt.Errorf("mark project job artifact stored: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectJobArtifactKey(id)); err != nil {
		return fmt.Errorf("delete project job artifact: %w", err)
	}
	return nil
}

func projectJobArtifactKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectJobArtifactSchema.ID, id))
}
