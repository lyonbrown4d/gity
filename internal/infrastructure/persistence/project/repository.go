package project

import (
	"context"
	"fmt"
	projectports "github.com/DaiYuANg/gity/internal/application/ports"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
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
	base *dbxrepo.Base[projectdomain.Project, dbschema.ProjectSchemaDef]
}

type CreateInput = projectports.CreateProjectInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[projectdomain.Project](db, dbschema.ProjectSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewProjectRepository(repo *Repository) projectports.ProjectRepository {
	return repo
}

func (r *Repository) List(ctx context.Context, namespaceID *int64) (*collectionx.List[projectdomain.Project], error) {
	query := querydsl.Select(dbschema.ProjectSchema.AllColumns().Values()...).
		From(dbschema.ProjectSchema).
		OrderBy(dbschema.ProjectSchema.ID.Desc())
	if namespaceID != nil {
		query = query.Where(dbschema.ProjectSchema.NamespaceID.Eq(*namespaceID))
	}
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	return persistence.One(r.base.GetByID(ctx, id))
}

func (r *Repository) GetByFullPath(ctx context.Context, fullPath string) (projectdomain.Project, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"full_path": strings.TrimSpace(fullPath),
	}))
}

func (r *Repository) Create(ctx context.Context, input CreateInput, namespace namespacedomain.Namespace) (projectdomain.Project, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	defaultBranch := strings.TrimSpace(input.DefaultBranch)
	if defaultBranch == "" {
		defaultBranch = "main"
	}
	visibility := strings.TrimSpace(input.Visibility)
	if visibility == "" {
		visibility = "private"
	}
	fullPath := namespace.FullPath + "/" + trimmedPath
	now := time.Now().UTC()

	item := projectdomain.Project{
		NamespaceID:   input.NamespaceID,
		Name:          trimmedName,
		PathKey:       trimmedPath,
		FullPath:      fullPath,
		Visibility:    visibility,
		Description:   strings.TrimSpace(input.Description),
		DefaultBranch: defaultBranch,
		CreatedAt:     now,
		UpdatedAt:     now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return projectdomain.Project{}, fmt.Errorf("insert project: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project: %w", err)
	}
	return nil
}
