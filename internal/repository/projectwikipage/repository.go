package projectwikipage

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
	base *dbxrepo.Base[entity.ProjectWikiPage, entity.ProjectWikiPageSchemaDef]
}

type CreateInput struct {
	ProjectID          int64
	Slug               string
	Title              string
	Content            string
	Format             string
	AuthorUserID       int64
	LastEditedByUserID int64
}

type UpdateInput struct {
	Title              *string
	Content            *string
	LastEditedByUserID int64
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectWikiPage](db, entity.ProjectWikiPageSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[entity.ProjectWikiPage], error) {
	query := querydsl.Select(entity.ProjectWikiPageSchema.AllColumns().Values()...).
		From(entity.ProjectWikiPageSchema).
		Where(entity.ProjectWikiPageSchema.ProjectID.Eq(projectID)).
		OrderBy(entity.ProjectWikiPageSchema.UpdatedAt.Desc(), entity.ProjectWikiPageSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndSlug(ctx context.Context, projectID int64, slug string) (entity.ProjectWikiPage, error) {
	query := querydsl.Select(entity.ProjectWikiPageSchema.AllColumns().Values()...).
		From(entity.ProjectWikiPageSchema).
		Where(querydsl.And(
			entity.ProjectWikiPageSchema.ProjectID.Eq(projectID),
			entity.ProjectWikiPageSchema.Slug.Eq(strings.TrimSpace(slug)),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectWikiPage, error) {
	now := time.Now().UTC()
	lastEditor := input.LastEditedByUserID
	if lastEditor == 0 {
		lastEditor = input.AuthorUserID
	}
	format := strings.TrimSpace(input.Format)
	if format == "" {
		format = "markdown"
	}
	item := entity.ProjectWikiPage{
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
		return entity.ProjectWikiPage{}, fmt.Errorf("insert project wiki page: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]querydsl.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, entity.ProjectWikiPageSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Content != nil {
		assignments = append(assignments, entity.ProjectWikiPageSchema.Content.Set(strings.TrimSpace(*input.Content)))
	}
	if input.LastEditedByUserID > 0 {
		assignments = append(assignments, entity.ProjectWikiPageSchema.LastEditedByUserID.Set(input.LastEditedByUserID))
	}
	assignments = append(assignments, entity.ProjectWikiPageSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update project wiki page: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project wiki page: %w", err)
	}
	return nil
}
