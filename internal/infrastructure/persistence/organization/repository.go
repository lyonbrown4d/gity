package organization

import (
	"context"
	"fmt"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	organizationports "github.com/lyonbrown4d/gity/internal/application/ports"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[organizationdomain.Organization, dbschema.OrganizationSchemaDef]
}

type CreateInput = organizationports.CreateOrganizationInput
type UpdateInput = organizationports.UpdateOrganizationInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[organizationdomain.Organization](db, dbschema.OrganizationSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewOrganizationRepository(repo *Repository) organizationports.OrganizationRepository {
	return repo
}

func (r *Repository) List(ctx context.Context) (*collectionx.List[organizationdomain.Organization], error) {
	query := querydsl.Select(dbschema.OrganizationSchema.AllColumns().Values()...).
		From(dbschema.OrganizationSchema).
		OrderBy(dbschema.OrganizationSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (organizationdomain.Organization, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.OrganizationSchema.ID).Get(ctx, id))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (organizationdomain.Organization, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	visibility := strings.TrimSpace(input.Visibility)
	if visibility == "" {
		visibility = "private"
	}
	now := time.Now().UTC()

	item := organizationdomain.Organization{
		Name:        trimmedName,
		PathKey:     trimmedPath,
		FullPath:    trimmedPath,
		Description: strings.TrimSpace(input.Description),
		Visibility:  visibility,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return organizationdomain.Organization{}, fmt.Errorf("insert organization: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := []querydsl.Assignment{}
	if input.Name != nil {
		assignments = append(assignments, dbschema.OrganizationSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.PathKey != nil {
		pathKey := strings.TrimSpace(*input.PathKey)
		assignments = append(assignments, dbschema.OrganizationSchema.PathKey.Set(pathKey), dbschema.OrganizationSchema.FullPath.Set(pathKey))
	}
	if input.Description != nil {
		assignments = append(assignments, dbschema.OrganizationSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.Visibility != nil {
		assignments = append(assignments, dbschema.OrganizationSchema.Visibility.Set(strings.TrimSpace(*input.Visibility)))
	}
	assignments = append(assignments, dbschema.OrganizationSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := dbxrepo.By(r.base, dbschema.OrganizationSchema.ID).Update(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update organization: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := dbxrepo.By(r.base, dbschema.OrganizationSchema.ID).Delete(ctx, id); err != nil {
		return fmt.Errorf("delete organization: %w", err)
	}
	return nil
}
