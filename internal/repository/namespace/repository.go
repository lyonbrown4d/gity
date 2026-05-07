package namespace

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
	base *dbxrepo.Base[entity.Namespace, entity.NamespaceSchemaDef]
}

type CreateInput struct {
	Kind        string
	Name        string
	PathKey     string
	Description string
}

type UpdateInput struct {
	Kind        *string
	Name        *string
	PathKey     *string
	Description *string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.Namespace](db, entity.NamespaceSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context) (*collectionx.List[entity.Namespace], error) {
	query := dbx.Select(entity.NamespaceSchema.AllColumns().Values()...).
		From(entity.NamespaceSchema).
		OrderBy(entity.NamespaceSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.Namespace, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.Namespace, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	now := time.Now().UTC()

	item := entity.Namespace{
		Kind:        strings.TrimSpace(input.Kind),
		Name:        trimmedName,
		PathKey:     trimmedPath,
		FullPath:    trimmedPath,
		Description: strings.TrimSpace(input.Description),
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.Namespace{}, fmt.Errorf("insert namespace: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := []dbx.Assignment{}
	if input.Kind != nil {
		assignments = append(assignments, entity.NamespaceSchema.Kind.Set(strings.TrimSpace(*input.Kind)))
	}
	if input.Name != nil {
		assignments = append(assignments, entity.NamespaceSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.PathKey != nil {
		pathKey := strings.TrimSpace(*input.PathKey)
		assignments = append(assignments, entity.NamespaceSchema.PathKey.Set(pathKey), entity.NamespaceSchema.FullPath.Set(pathKey))
	}
	if input.Description != nil {
		assignments = append(assignments, entity.NamespaceSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	assignments = append(assignments, entity.NamespaceSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update namespace: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete namespace: %w", err)
	}
	return nil
}
