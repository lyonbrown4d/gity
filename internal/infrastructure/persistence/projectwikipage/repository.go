package projectwikipage

import (
	"context"
	"fmt"
	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[wikidomain.ProjectWikiPage, wikidomain.ProjectWikiPageSchemaDef]
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
	return &Repository{base: dbxrepo.NewWithOptions[wikidomain.ProjectWikiPage](db, wikidomain.ProjectWikiPageSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[wikidomain.ProjectWikiPage], error) {
	query := querydsl.Select(wikidomain.ProjectWikiPageSchema.AllColumns().Values()...).
		From(wikidomain.ProjectWikiPageSchema).
		Where(wikidomain.ProjectWikiPageSchema.ProjectID.Eq(projectID)).
		OrderBy(wikidomain.ProjectWikiPageSchema.UpdatedAt.Desc(), wikidomain.ProjectWikiPageSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndSlug(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error) {
	query := querydsl.Select(wikidomain.ProjectWikiPageSchema.AllColumns().Values()...).
		From(wikidomain.ProjectWikiPageSchema).
		Where(querydsl.And(
			wikidomain.ProjectWikiPageSchema.ProjectID.Eq(projectID),
			wikidomain.ProjectWikiPageSchema.Slug.Eq(strings.TrimSpace(slug)),
		)).
		Limit(1)
	return r.base.First(ctx, query)
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
		assignments = append(assignments, wikidomain.ProjectWikiPageSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Content != nil {
		assignments = append(assignments, wikidomain.ProjectWikiPageSchema.Content.Set(strings.TrimSpace(*input.Content)))
	}
	if input.LastEditedByUserID > 0 {
		assignments = append(assignments, wikidomain.ProjectWikiPageSchema.LastEditedByUserID.Set(input.LastEditedByUserID))
	}
	assignments = append(assignments, wikidomain.ProjectWikiPageSchema.UpdatedAt.Set(time.Now().UTC()))
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
