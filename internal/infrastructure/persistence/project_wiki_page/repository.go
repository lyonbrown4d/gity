package projectwikipage

import (
	"context"
	"fmt"
	wikiports "github.com/DaiYuANg/gity/internal/application/ports"
	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[wikidomain.ProjectWikiPage, dbschema.ProjectWikiPageSchemaDef]
}

type CreateInput = wikiports.CreateProjectWikiPageInput

type UpdateInput = wikiports.UpdateProjectWikiPageInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[wikidomain.ProjectWikiPage](db, dbschema.ProjectWikiPageSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectWikiPageRepository(repo *Repository) wikiports.ProjectWikiPageRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[wikidomain.ProjectWikiPage], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectWikiPageSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectWikiPageSchema.UpdatedAt.Desc(), dbschema.ProjectWikiPageSchema.ID.Desc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndSlug(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectWikiPageSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectWikiPageSchema.Slug.Eq(strings.TrimSpace(slug))).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (wikidomain.ProjectWikiPage, error) {
	now := time.Now().UTC()
	lastEditor := input.LastEditedByUserID
	if lastEditor == 0 {
		lastEditor = input.AuthorUserID
	}
	format := strings.TrimSpace(input.Format)
	if format == "" {
		format = "markdown"
	}
	item := wikidomain.ProjectWikiPage{
		ProjectID:          input.ProjectID,
		Slug:               strings.TrimSpace(input.Slug),
		Title:              strings.TrimSpace(input.Title),
		Content:            strings.TrimSpace(input.Content),
		Format:             format,
		AuthorUserID:       input.AuthorUserID,
		LastEditedByUserID: lastEditor,
		CreatedAt:          now,
		UpdatedAt:          now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return wikidomain.ProjectWikiPage{}, fmt.Errorf("insert project wiki page: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]querydsl.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, dbschema.ProjectWikiPageSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Content != nil {
		assignments = append(assignments, dbschema.ProjectWikiPageSchema.Content.Set(strings.TrimSpace(*input.Content)))
	}
	if input.LastEditedByUserID > 0 {
		assignments = append(assignments, dbschema.ProjectWikiPageSchema.LastEditedByUserID.Set(input.LastEditedByUserID))
	}
	assignments = append(assignments, dbschema.ProjectWikiPageSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := dbxrepo.PatchSet(r.base, projectWikiPageKey(id)).Set(assignments...).Apply(ctx); err != nil {
		return fmt.Errorf("update project wiki page: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectWikiPageKey(id)); err != nil {
		return fmt.Errorf("delete project wiki page: %w", err)
	}
	return nil
}

func projectWikiPageKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectWikiPageSchema.ID, id))
}
