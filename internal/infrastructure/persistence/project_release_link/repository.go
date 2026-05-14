package projectreleaselink

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	releaseports "github.com/lyonbrown4d/gity/internal/application/ports"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[releasedomain.ProjectReleaseLink, dbschema.ProjectReleaseLinkSchemaDef]
}

type CreateInput = releaseports.CreateProjectReleaseLinkInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[releasedomain.ProjectReleaseLink](db, dbschema.ProjectReleaseLinkSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectReleaseLinkRepository(repo *Repository) releaseports.ProjectReleaseLinkRepository {
	return repo
}

func (r *Repository) ListByReleaseID(ctx context.Context, releaseID int64) (*collectionx.List[releasedomain.ProjectReleaseLink], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectReleaseLinkSchema.ProjectReleaseID.Eq(releaseID)).
		OrderBy(dbschema.ProjectReleaseLinkSchema.ID.Asc()).
		List(ctx))
}

func (r *Repository) ListByReleaseIDs(ctx context.Context, releaseIDs ...int64) (*collectionx.List[releasedomain.ProjectReleaseLink], error) {
	if len(releaseIDs) == 0 {
		return collectionx.NewList[releasedomain.ProjectReleaseLink](), nil
	}
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectReleaseLinkSchema.ProjectReleaseID.In(releaseIDs...)).
		OrderBy(dbschema.ProjectReleaseLinkSchema.ProjectReleaseID.Asc(), dbschema.ProjectReleaseLinkSchema.ID.Asc()).
		List(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (releasedomain.ProjectReleaseLink, error) {
	now := time.Now().UTC()
	item := releasedomain.ProjectReleaseLink{
		ProjectReleaseID: input.ProjectReleaseID,
		Name:             strings.TrimSpace(input.Name),
		URL:              strings.TrimSpace(input.URL),
		LinkType:         normalizeLinkType(input.LinkType),
		CreatedAt:        now,
		UpdatedAt:        now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return releasedomain.ProjectReleaseLink{}, fmt.Errorf("insert project release link: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectReleaseLinkKey(id)); err != nil {
		return fmt.Errorf("delete project release link: %w", err)
	}
	return nil
}

func normalizeLinkType(value string) string {
	value = strings.TrimSpace(value)
	if value == "" {
		return "other"
	}
	return value
}

func projectReleaseLinkKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectReleaseLinkSchema.ID, id))
}
