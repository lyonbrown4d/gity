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
		base: dbxrepo.NewWithOptions[projectdomain.Project](db, dbschema.ProjectSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectRepository(repo *Repository) projectports.ProjectRepository {
	return repo
}

func (r *Repository) List(ctx context.Context, organizationID *int64) (*collectionx.List[projectdomain.Project], error) {
	query := querydsl.Select(dbschema.ProjectSchema.AllColumns().Values()...).
		From(dbschema.ProjectSchema).
		Where(activeProjectPredicate()).
		OrderBy(dbschema.ProjectSchema.ID.Desc())
	if organizationID != nil {
		query = query.Where(querydsl.And(activeProjectPredicate(), dbschema.ProjectSchema.OrganizationID.Eq(*organizationID)))
	}
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	return r.getByID(ctx, id, false)
}

func (r *Repository) GetIncludingDeletedByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	return r.getByID(ctx, id, true)
}

func (r *Repository) GetByFullPath(ctx context.Context, fullPath string) (projectdomain.Project, error) {
	query := querydsl.Select(dbschema.ProjectSchema.AllColumns().Values()...).
		From(dbschema.ProjectSchema).
		Where(querydsl.And(
			dbschema.ProjectSchema.FullPath.Eq(strings.TrimSpace(fullPath)),
			activeProjectPredicate(),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
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
	if _, err := dbxrepo.By(r.base, dbschema.ProjectSchema.ID).Update(ctx, id,
		dbschema.ProjectSchema.Status.Set(projectdomain.ProjectStatusPendingDelete),
		dbschema.ProjectSchema.DeletedAt.Set(deletedAt),
		dbschema.ProjectSchema.UpdatedAt.Set(deletedAt),
	); err != nil {
		return fmt.Errorf("mark project pending delete: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := dbxrepo.By(r.base, dbschema.ProjectSchema.ID).Delete(ctx, id); err != nil {
		return fmt.Errorf("delete project: %w", err)
	}
	return nil
}

func (r *Repository) getByID(ctx context.Context, id int64, includeDeleted bool) (projectdomain.Project, error) {
	query := querydsl.Select(dbschema.ProjectSchema.AllColumns().Values()...).
		From(dbschema.ProjectSchema).
		Where(dbschema.ProjectSchema.ID.Eq(id)).
		Limit(1)
	if !includeDeleted {
		query = query.Where(querydsl.And(dbschema.ProjectSchema.ID.Eq(id), activeProjectPredicate()))
	}
	return persistence.One(r.base.First(ctx, query))
}

func activeProjectPredicate() querydsl.Predicate {
	return dbschema.ProjectSchema.Status.Eq(projectdomain.ProjectStatusActive)
}
