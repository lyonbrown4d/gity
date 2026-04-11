package namespace

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
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

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.Namespace](db, entity.NamespaceSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context) (collectionx.List[entity.Namespace], error) {
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
