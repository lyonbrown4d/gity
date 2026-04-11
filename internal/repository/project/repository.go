package project

import (
	"context"
	"database/sql"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	base *dbxrepo.Base[entity.Project, entity.ProjectSchemaDef]
}

type CreateInput struct {
	NamespaceID   int64
	Name          string
	PathKey       string
	Description   string
	DefaultBranch string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.Project](db, entity.ProjectSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context, namespaceID sql.NullInt64) (collectionx.List[entity.Project], error) {
	query := dbx.Select(entity.ProjectSchema.AllColumns().Values()...).
		From(entity.ProjectSchema).
		OrderBy(entity.ProjectSchema.ID.Desc())
	if namespaceID.Valid {
		query = query.Where(entity.ProjectSchema.NamespaceID.Eq(namespaceID.Int64))
	}
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.Project, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput, namespace entity.Namespace) (entity.Project, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	defaultBranch := strings.TrimSpace(input.DefaultBranch)
	if defaultBranch == "" {
		defaultBranch = "main"
	}
	fullPath := namespace.FullPath + "/" + trimmedPath
	now := time.Now().UTC()

	item := entity.Project{
		NamespaceID:   input.NamespaceID,
		Name:          trimmedName,
		PathKey:       trimmedPath,
		FullPath:      fullPath,
		Description:   strings.TrimSpace(input.Description),
		DefaultBranch: defaultBranch,
		CreatedAt:     now,
		UpdatedAt:     now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.Project{}, fmt.Errorf("insert project: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project: %w", err)
	}
	return nil
}
