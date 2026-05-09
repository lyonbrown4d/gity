package project

import (
	"context"
	"fmt"
	projectports "github.com/DaiYuANg/gity/internal/application/ports"
	organizationdomain "github.com/DaiYuANg/gity/internal/domain/organization"
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
		base: dbxrepo.NewWithOptions[projectdomain.Project](
			db,
			dbschema.ProjectSchema,
			dbxrepo.WithKeyNotFoundAsError(true),
			dbxrepo.WithDefaultSpecs(dbxrepo.Where(activeProjectPredicate())),
		),
	}, nil
}

func NewProjectRepository(repo *Repository) projectports.ProjectRepository {
	return repo
}

func (r *Repository) List(ctx context.Context, organizationID *int64) (*collectionx.List[projectdomain.Project], error) {
	query := dbxrepo.Query(r.base).
		OrderBy(dbschema.ProjectSchema.ID.Desc())
	if organizationID != nil {
		query = query.Where(dbschema.ProjectSchema.OrganizationID.Eq(*organizationID))
	}
	return persistence.Many(query.List(ctx))
}

func (r *Repository) Batch(ctx context.Context, organizationID *int64, size int, handle func(*collectionx.List[projectdomain.Project]) error) error {
	query := dbxrepo.Query(r.base).
		OrderBy(dbschema.ProjectSchema.ID.Asc())
	if organizationID != nil {
		query = query.Where(dbschema.ProjectSchema.OrganizationID.Eq(*organizationID))
	}
	if err := query.Batch(ctx, size, handle); err != nil {
		return fmt.Errorf("batch projects: %w", err)
	}
	return nil
}

func (r *Repository) GetByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	return r.getByID(ctx, id, false)
}

func (r *Repository) GetIncludingDeletedByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	return r.getByID(ctx, id, true)
}

func (r *Repository) GetByFullPath(ctx context.Context, fullPath string) (projectdomain.Project, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectSchema.FullPath.Eq(strings.TrimSpace(fullPath))).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput, organization organizationdomain.Organization) (projectdomain.Project, error) {
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
	fullPath := organization.FullPath + "/" + trimmedPath
	now := time.Now().UTC()

	item := projectdomain.Project{
		OrganizationID: input.OrganizationID,
		Name:           trimmedName,
		PathKey:        trimmedPath,
		FullPath:       fullPath,
		Visibility:     visibility,
		Description:    strings.TrimSpace(input.Description),
		DefaultBranch:  defaultBranch,
		Status:         projectdomain.ProjectStatusActive,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return projectdomain.Project{}, fmt.Errorf("insert project: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkPendingDeleteByID(ctx context.Context, id int64, deletedAt time.Time) error {
	if deletedAt.IsZero() {
		deletedAt = time.Now().UTC()
	} else {
		deletedAt = deletedAt.UTC()
	}
	if _, err := dbxrepo.PatchSet(r.base, projectKey(id)).Set(
		dbschema.ProjectSchema.Status.Set(projectdomain.ProjectStatusPendingDelete),
		dbschema.ProjectSchema.DeletedAt.Set(deletedAt),
		dbschema.ProjectSchema.UpdatedAt.Set(deletedAt),
	).Apply(ctx); err != nil {
		return fmt.Errorf("mark project pending delete: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByKeySet(ctx, projectKey(id)); err != nil {
		return fmt.Errorf("delete project: %w", err)
	}
	return nil
}

func (r *Repository) getByID(ctx context.Context, id int64, includeDeleted bool) (projectdomain.Project, error) {
	query := dbxrepo.Query(r.base).Where(dbschema.ProjectSchema.ID.Eq(id))
	if includeDeleted {
		query = query.WithDeleted()
	}
	return persistence.One(query.First(ctx))
}

func activeProjectPredicate() querydsl.Predicate {
	return dbschema.ProjectSchema.Status.Eq(projectdomain.ProjectStatusActive)
}

func projectKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectSchema.ID, id))
}
