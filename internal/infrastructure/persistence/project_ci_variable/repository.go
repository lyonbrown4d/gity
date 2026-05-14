package projectcivariable

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	ciports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbaudit "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_audit"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectCIVariable, dbschema.ProjectCIVariableSchemaDef]
}

type UpsertInput = ciports.UpsertProjectCIVariableInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectCIVariable](
			db,
			dbschema.ProjectCIVariableSchema,
			dbxrepo.WithKeyNotFoundAsError(true),
			dbxrepo.WithAuditWriter(dbaudit.ProjectCIVariableAudit()),
		),
	}, nil
}

func NewProjectCIVariableRepository(repo *Repository) ciports.ProjectCIVariableRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectCIVariable], error) {
	query := querydsl.Select(dbschema.ProjectCIVariableSchema.AllColumns().Values()...).
		From(dbschema.ProjectCIVariableSchema).
		Where(dbschema.ProjectCIVariableSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectCIVariableSchema.Key.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndKey(ctx context.Context, projectID int64, key string) (cidomain.ProjectCIVariable, error) {
	query := querydsl.Select(dbschema.ProjectCIVariableSchema.AllColumns().Values()...).
		From(dbschema.ProjectCIVariableSchema).
		Where(querydsl.And(
			dbschema.ProjectCIVariableSchema.ProjectID.Eq(projectID),
			dbschema.ProjectCIVariableSchema.Key.Eq(normalizeVariableKey(key)),
		)).
		Limit(1)
	item, err := r.base.First(ctx, query)
	if err != nil {
		if persistence.IsNotFound(err) {
			return cidomain.ProjectCIVariable{}, ciports.ErrNotFound
		}
		return cidomain.ProjectCIVariable{}, oops.In("persistence.project_ci_variable").With("project_id", projectID, "key", key).Wrapf(err, "load project ci variable")
	}
	return item, nil
}

func (r *Repository) Upsert(ctx context.Context, input UpsertInput) (cidomain.ProjectCIVariable, error) {
	key := normalizeVariableKey(input.Key)
	existing, err := r.GetByProjectAndKey(ctx, input.ProjectID, key)
	if err == nil {
		if updateErr := r.updateByID(ctx, existing.ID, input); updateErr != nil {
			return cidomain.ProjectCIVariable{}, updateErr
		}
		return r.GetByProjectAndKey(ctx, input.ProjectID, key)
	}
	if !errors.Is(err, ciports.ErrNotFound) {
		return cidomain.ProjectCIVariable{}, err
	}
	now := time.Now().UTC()
	item := cidomain.ProjectCIVariable{
		ProjectID: input.ProjectID,
		Key:       key,
		Value:     input.Value,
		Masked:    input.Masked,
		Protected: input.Protected,
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return cidomain.ProjectCIVariable{}, oops.In("persistence.project_ci_variable").With("project_id", input.ProjectID, "key", input.Key).Wrapf(err, "insert project ci variable")
	}
	return item, nil
}

func (r *Repository) DeleteByProjectAndKey(ctx context.Context, projectID int64, key string) error {
	item, err := r.GetByProjectAndKey(ctx, projectID, key)
	if err != nil {
		if errors.Is(err, ciports.ErrNotFound) {
			return nil
		}
		return err
	}
	if _, err := r.base.DeleteByKeySet(ctx, projectCIVariableKey(item.ID)); err != nil {
		return fmt.Errorf("delete project ci variable: %w", err)
	}
	return nil
}

func (r *Repository) updateByID(ctx context.Context, id int64, input UpsertInput) error {
	if _, err := dbxrepo.PatchSet(r.base, projectCIVariableKey(id)).Set(
		dbschema.ProjectCIVariableSchema.Value.Set(input.Value),
		dbschema.ProjectCIVariableSchema.Masked.Set(input.Masked),
		dbschema.ProjectCIVariableSchema.Protected.Set(input.Protected),
		dbschema.ProjectCIVariableSchema.UpdatedAt.Set(time.Now().UTC()),
	).Apply(ctx); err != nil {
		return fmt.Errorf("update project ci variable: %w", err)
	}
	return nil
}

func normalizeVariableKey(value string) string {
	return strings.ToUpper(strings.TrimSpace(value))
}

func projectCIVariableKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectCIVariableSchema.ID, id))
}
