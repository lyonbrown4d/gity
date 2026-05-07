package namespace

import (
	"context"
	"fmt"
	namespaceports "github.com/DaiYuANg/gity/internal/application/ports"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
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
	base *dbxrepo.Base[namespacedomain.Namespace, dbschema.NamespaceSchemaDef]
}

type CreateInput = namespaceports.CreateNamespaceInput
type UpdateInput = namespaceports.UpdateNamespaceInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[namespacedomain.Namespace](db, dbschema.NamespaceSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewNamespaceRepository(repo *Repository) namespaceports.NamespaceRepository {
	return repo
}

func (r *Repository) List(ctx context.Context) (*collectionx.List[namespacedomain.Namespace], error) {
	query := querydsl.Select(dbschema.NamespaceSchema.AllColumns().Values()...).
		From(dbschema.NamespaceSchema).
		OrderBy(dbschema.NamespaceSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (namespacedomain.Namespace, error) {
	return persistence.One(r.base.GetByID(ctx, id))
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
		assignments = append(assignments, dbschema.NamespaceSchema.Kind.Set(strings.TrimSpace(*input.Kind)))
	}
	if input.Name != nil {
		assignments = append(assignments, dbschema.NamespaceSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.PathKey != nil {
		pathKey := strings.TrimSpace(*input.PathKey)
		assignments = append(assignments, dbschema.NamespaceSchema.PathKey.Set(pathKey), dbschema.NamespaceSchema.FullPath.Set(pathKey))
	}
	if input.Description != nil {
		assignments = append(assignments, dbschema.NamespaceSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	assignments = append(assignments, dbschema.NamespaceSchema.UpdatedAt.Set(time.Now().UTC()))
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
