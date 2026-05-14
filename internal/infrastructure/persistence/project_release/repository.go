package projectrelease

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	releaseports "github.com/lyonbrown4d/gity/internal/application/ports"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[releasedomain.ProjectRelease, dbschema.ProjectReleaseSchemaDef]
}

type CreateInput = releaseports.CreateProjectReleaseInput
type UpdateInput = releaseports.UpdateProjectReleaseInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[releasedomain.ProjectRelease](db, dbschema.ProjectReleaseSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectReleaseRepository(repo *Repository) releaseports.ProjectReleaseRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[releasedomain.ProjectRelease], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectReleaseSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectReleaseSchema.ReleasedAt.Desc(), dbschema.ProjectReleaseSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (releasedomain.ProjectRelease, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectReleaseSchema.ID).Get(ctx, id))
}

func (r *Repository) GetByProjectAndTagName(ctx context.Context, projectID int64, tagName string) (releasedomain.ProjectRelease, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectReleaseSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectReleaseSchema.TagName.Eq(strings.TrimSpace(tagName))).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (releasedomain.ProjectRelease, error) {
	now := time.Now().UTC()
	releasedAt := input.ReleasedAt
	if releasedAt.IsZero() {
		releasedAt = now
	}
	item := releasedomain.ProjectRelease{
		ProjectID:       input.ProjectID,
		TagName:         strings.TrimSpace(input.TagName),
		Name:            strings.TrimSpace(input.Name),
		Description:     strings.TrimSpace(input.Description),
		CreatedByUserID: input.CreatedByUserID,
		ReleasedAt:      releasedAt.UTC(),
		CreatedAt:       now,
		UpdatedAt:       now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return releasedomain.ProjectRelease{}, fmt.Errorf("insert project release: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]querydsl.Assignment, 0, 4)
	if input.Name != nil {
		assignments = append(assignments, dbschema.ProjectReleaseSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.Description != nil {
		assignments = append(assignments, dbschema.ProjectReleaseSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.ReleasedAt != nil {
		assignments = append(assignments, dbschema.ProjectReleaseSchema.ReleasedAt.Set(input.ReleasedAt.UTC()))
	}
	assignments = append(assignments, dbschema.ProjectReleaseSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := dbxrepo.PatchSet(r.base, projectReleaseKey(id)).Set(assignments...).Apply(ctx); err != nil {
		return fmt.Errorf("update project release: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectReleaseKey(id)); err != nil {
		return fmt.Errorf("delete project release: %w", err)
	}
	return nil
}

func projectReleaseKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectReleaseSchema.ID, id))
}
