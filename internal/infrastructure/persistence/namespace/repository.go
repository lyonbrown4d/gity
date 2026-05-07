package namespace

import (
	"context"
	"fmt"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[namespacedomain.Namespace, namespacedomain.NamespaceSchemaDef]
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
		base: dbxrepo.NewWithOptions[namespacedomain.Namespace](db, namespacedomain.NamespaceSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context) (*collectionx.List[namespacedomain.Namespace], error) {
	query := querydsl.Select(namespacedomain.NamespaceSchema.AllColumns().Values()...).
		From(namespacedomain.NamespaceSchema).
		OrderBy(namespacedomain.NamespaceSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (namespacedomain.Namespace, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (namespacedomain.Namespace, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	now := time.Now().UTC()

	item := namespacedomain.Namespace{
		Kind:        strings.TrimSpace(input.Kind),
		Name:        trimmedName,
		PathKey:     trimmedPath,
		FullPath:    trimmedPath,
		Description: strings.TrimSpace(input.Description),
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return namespacedomain.Namespace{}, fmt.Errorf("insert namespace: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := []querydsl.Assignment{}
	if input.Kind != nil {
		assignments = append(assignments, namespacedomain.NamespaceSchema.Kind.Set(strings.TrimSpace(*input.Kind)))
	}
	if input.Name != nil {
		assignments = append(assignments, namespacedomain.NamespaceSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.PathKey != nil {
		pathKey := strings.TrimSpace(*input.PathKey)
		assignments = append(assignments, namespacedomain.NamespaceSchema.PathKey.Set(pathKey), namespacedomain.NamespaceSchema.FullPath.Set(pathKey))
	}
	if input.Description != nil {
		assignments = append(assignments, namespacedomain.NamespaceSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	assignments = append(assignments, namespacedomain.NamespaceSchema.UpdatedAt.Set(time.Now().UTC()))
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
