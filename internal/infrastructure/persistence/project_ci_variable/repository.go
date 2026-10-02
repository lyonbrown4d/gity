package projectcivariable

import (
	"context"
	"errors"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/idgen"
	mapperx "github.com/arcgolabs/dbx/mapper"
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
	db          *dbx.DB
	base        *dbxrepo.Base[cidomain.ProjectCIVariable, dbschema.ProjectCIVariableSchemaDef]
	mapper      mapperx.StructMapper[cidomain.ProjectCIVariable]
	auditWriter dbxrepo.AuditWriter
}

type UpsertInput = ciports.UpsertProjectCIVariableInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	mapper, err := mapperx.NewStructMapper[cidomain.ProjectCIVariable]()
	if err != nil {
		return nil, oops.In("persistence.project_ci_variable").Wrapf(err, "initialize project ci variable mapper")
	}
	auditWriter := dbaudit.ProjectCIVariableAudit()
	return &Repository{
		db: db,
		base: dbxrepo.NewWithOptions[cidomain.ProjectCIVariable](
			db,
			dbschema.ProjectCIVariableSchema,
			dbxrepo.WithKeyNotFoundAsError(true),
			dbxrepo.WithAuditWriter(auditWriter),
		),
		mapper:      mapper,
		auditWriter: auditWriter,
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
	item, found, err := r.findByProjectAndKey(ctx, projectID, key)
	if err != nil {
		return cidomain.ProjectCIVariable{}, oops.In("persistence.project_ci_variable").With("project_id", projectID, "key", key).Wrapf(err, "load project ci variable")
	}
	if !found {
		return cidomain.ProjectCIVariable{}, ciports.ErrNotFound
	}
	return item, nil
}

func (r *Repository) Upsert(ctx context.Context, input UpsertInput) (cidomain.ProjectCIVariable, error) {
	if r == nil || r.db == nil {
		return cidomain.ProjectCIVariable{}, oops.In("persistence.project_ci_variable").Wrapf(dbx.ErrNilDB, "upsert project ci variable")
	}
	id, err := r.nextID(ctx, input)
	if err != nil {
		return cidomain.ProjectCIVariable{}, err
	}

	input.Key = normalizeVariableKey(input.Key)
	now := time.Now().UTC()
	query := projectCIVariableUpsertQuery(id, input, now)
	if querydsl.DialectFeatures(r.db.Dialect()).SupportsReturning {
		return r.upsertReturning(ctx, input, query)
	}
	return r.upsertAndLoad(ctx, input, query)
}

func (r *Repository) nextID(ctx context.Context, input UpsertInput) (int64, error) {
	generatedID, err := r.db.IDGenerator().GenerateID(ctx, idgen.Request{Strategy: idgen.StrategySnowflake})
	if err != nil {
		return 0, oops.In("persistence.project_ci_variable").With("project_id", input.ProjectID, "key", input.Key).Wrapf(err, "generate project ci variable id")
	}
	id, ok := generatedID.(int64)
	if !ok {
		return 0, oops.In("persistence.project_ci_variable").New("generate project ci variable id: unexpected id type")
	}
	return id, nil
}

func (r *Repository) upsertReturning(ctx context.Context, input UpsertInput, query *querydsl.InsertQuery) (cidomain.ProjectCIVariable, error) {
	var item cidomain.ProjectCIVariable
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, _ *dbxrepo.Base[cidomain.ProjectCIVariable, dbschema.ProjectCIVariableSchemaDef]) error {
		created, queryErr := dbx.QueryOne[cidomain.ProjectCIVariable](
			ctx,
			tx,
			query.Returning(dbschema.ProjectCIVariableSchema.AllColumns().Values()...),
			r.mapper,
		)
		if queryErr != nil {
			return oops.In("persistence.project_ci_variable").Wrapf(queryErr, "execute returning project ci variable upsert")
		}
		item = created
		if auditErr := r.auditWriter.WriteAudit(ctx, tx, dbxrepo.AuditOperationUpsert, item); auditErr != nil {
			return oops.In("persistence.project_ci_variable").Wrapf(auditErr, "write project ci variable upsert audit")
		}
		return nil
	})
	if err != nil {
		return cidomain.ProjectCIVariable{}, r.wrapUpsertError(err, input)
	}
	return item, nil
}

func (r *Repository) upsertAndLoad(ctx context.Context, input UpsertInput, query *querydsl.InsertQuery) (cidomain.ProjectCIVariable, error) {
	var item cidomain.ProjectCIVariable
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, txRepo *dbxrepo.Base[cidomain.ProjectCIVariable, dbschema.ProjectCIVariableSchemaDef]) error {
		if _, execErr := dbx.Exec(ctx, tx, query); execErr != nil {
			return oops.In("persistence.project_ci_variable").Wrapf(execErr, "execute project ci variable upsert")
		}
		loaded, found, findErr := findByProjectAndKey(ctx, txRepo, input.ProjectID, input.Key)
		if findErr != nil {
			return findErr
		}
		if !found {
			return oops.In("persistence.project_ci_variable").With("project_id", input.ProjectID, "key", input.Key).New("load upserted project ci variable: row not found")
		}
		item = loaded
		if auditErr := r.auditWriter.WriteAudit(ctx, tx, dbxrepo.AuditOperationUpsert, item); auditErr != nil {
			return oops.In("persistence.project_ci_variable").Wrapf(auditErr, "write project ci variable upsert audit")
		}
		return nil
	})
	if err != nil {
		return cidomain.ProjectCIVariable{}, r.wrapUpsertError(err, input)
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
		return oops.In("persistence.project_ci_variable").With("project_id", projectID, "key", key).Wrapf(err, "delete project ci variable")
	}
	return nil
}

func (r *Repository) findByProjectAndKey(ctx context.Context, projectID int64, key string) (cidomain.ProjectCIVariable, bool, error) {
	return findByProjectAndKey(ctx, r.base, projectID, key)
}

func findByProjectAndKey(
	ctx context.Context,
	base *dbxrepo.Base[cidomain.ProjectCIVariable, dbschema.ProjectCIVariableSchemaDef],
	projectID int64,
	key string,
) (cidomain.ProjectCIVariable, bool, error) {
	query := querydsl.Select(dbschema.ProjectCIVariableSchema.AllColumns().Values()...).
		From(dbschema.ProjectCIVariableSchema).
		Where(querydsl.And(
			dbschema.ProjectCIVariableSchema.ProjectID.Eq(projectID),
			dbschema.ProjectCIVariableSchema.Key.Eq(normalizeVariableKey(key)),
		)).
		Limit(1)
	item, err := base.FirstOption(ctx, query)
	if err != nil {
		return cidomain.ProjectCIVariable{}, false, oops.In("persistence.project_ci_variable").Wrapf(err, "query project ci variable")
	}
	value, found := item.Get()
	return value, found, nil
}

func (r *Repository) wrapUpsertError(err error, input UpsertInput) error {
	return oops.In("persistence.project_ci_variable").With("project_id", input.ProjectID, "key", input.Key).Wrapf(err, "upsert project ci variable")
}

func projectCIVariableUpsertQuery(id int64, input UpsertInput, now time.Time) *querydsl.InsertQuery {
	schema := dbschema.ProjectCIVariableSchema
	return querydsl.InsertInto(schema).Values(
		schema.ID.Set(id),
		schema.ProjectID.Set(input.ProjectID),
		schema.Key.Set(input.Key),
		schema.Value.Set(input.Value),
		schema.Masked.Set(input.Masked),
		schema.Protected.Set(input.Protected),
		schema.CreatedAt.Set(now),
		schema.UpdatedAt.Set(now),
	).OnConflict(schema.ProjectID, schema.Key).DoUpdateSet(
		schema.Value.SetExcluded(),
		schema.Masked.SetExcluded(),
		schema.Protected.SetExcluded(),
		schema.UpdatedAt.SetExcluded(),
	)
}

func normalizeVariableKey(value string) string {
	return strings.ToUpper(strings.TrimSpace(value))
}

func projectCIVariableKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectCIVariableSchema.ID, id))
}
