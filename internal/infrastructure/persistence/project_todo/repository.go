package projecttodo

import (
	"context"
	"database/sql"
	"errors"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/idgen"
	mapperx "github.com/arcgolabs/dbx/mapper"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	todoports "github.com/lyonbrown4d/gity/internal/application/ports"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

type Repository struct {
	db     *dbx.DB
	base   *dbxrepo.Base[tododomain.ProjectTodo, dbschema.ProjectTodoSchemaDef]
	mapper mapperx.StructMapper[projectTodoRow]
}

type projectTodoRow struct {
	ID             int64        `dbx:"id"`
	UserID         int64        `dbx:"user_id"`
	OrganizationID int64        `dbx:"organization_id"`
	ProjectID      int64        `dbx:"project_id"`
	Kind           string       `dbx:"kind"`
	State          string       `dbx:"state"`
	TargetType     string       `dbx:"target_type"`
	TargetID       string       `dbx:"target_id"`
	Title          string       `dbx:"title"`
	Summary        string       `dbx:"summary"`
	ActionURL      string       `dbx:"action_url"`
	CreatedAt      time.Time    `dbx:"created_at"`
	UpdatedAt      time.Time    `dbx:"updated_at"`
	DoneAt         sql.NullTime `dbx:"done_at"`
}

type CreateInput = todoports.CreateProjectTodoInput
type ListInput = todoports.ListProjectTodosInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	mapper, err := mapperx.NewStructMapper[projectTodoRow]()
	if err != nil {
		return nil, oops.In("persistence.todo").Wrapf(err, "initialize project todo mapper")
	}
	return &Repository{
		db:     db,
		base:   dbxrepo.NewWithOptions[tododomain.ProjectTodo](db, dbschema.ProjectTodoSchema, dbxrepo.WithKeyNotFoundAsError(true)),
		mapper: mapper,
	}, nil
}

func NewProjectTodoRepository(repo *Repository) todoports.ProjectTodoRepository {
	return repo
}

func (r *Repository) CreateIfNotExists(ctx context.Context, input CreateInput) (tododomain.ProjectTodo, error) {
	if r == nil || r.db == nil {
		return tododomain.ProjectTodo{}, r.wrapCreateError(dbx.ErrNilDB, input)
	}

	input.Kind = strings.TrimSpace(input.Kind)
	input.TargetType = strings.TrimSpace(input.TargetType)
	input.TargetID = strings.TrimSpace(input.TargetID)
	now := time.Now().UTC()
	item := tododomain.ProjectTodo{
		UserID:         input.UserID,
		OrganizationID: input.OrganizationID,
		ProjectID:      input.ProjectID,
		Kind:           input.Kind,
		State:          tododomain.ProjectTodoStatePending,
		TargetType:     input.TargetType,
		TargetID:       input.TargetID,
		Title:          strings.TrimSpace(input.Title),
		Summary:        strings.TrimSpace(input.Summary),
		ActionURL:      strings.TrimSpace(input.ActionURL),
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	generatedID, generateErr := r.db.IDGenerator().GenerateID(ctx, idgen.Request{Strategy: idgen.StrategySnowflake})
	if generateErr != nil {
		return tododomain.ProjectTodo{}, r.wrapCreateError(generateErr, input)
	}
	id, ok := generatedID.(int64)
	if !ok {
		return tododomain.ProjectTodo{}, r.wrapCreateError(errors.New("generate project todo id: unexpected id type"), input)
	}
	item.ID = id

	query := projectTodoUpsertQuery(item)
	if querydsl.DialectFeatures(r.db.Dialect()).SupportsReturning {
		created, queryErr := dbx.QueryOne[projectTodoRow](
			ctx,
			r.db,
			query.Returning(dbschema.ProjectTodoSchema.AllColumns().Values()...),
			r.mapper,
		)
		if queryErr != nil {
			return tododomain.ProjectTodo{}, r.wrapCreateError(queryErr, input)
		}
		return created.domain(), nil
	}
	if _, execErr := dbx.Exec(ctx, r.db, query); execErr != nil {
		return tododomain.ProjectTodo{}, r.wrapCreateError(execErr, input)
	}
	created, findErr := r.findExisting(ctx, input)
	if findErr != nil {
		return tododomain.ProjectTodo{}, r.wrapCreateError(findErr, input)
	}
	return created, nil
}

func (r *Repository) ListByUserID(ctx context.Context, input ListInput) (*collectionx.List[tododomain.ProjectTodo], error) {
	if input.Limit <= 0 || input.Limit > 200 {
		input.Limit = 50
	}
	state := strings.TrimSpace(input.State)
	query := querydsl.Select(dbschema.ProjectTodoSchema.AllColumns().Values()...).
		From(dbschema.ProjectTodoSchema).
		Where(dbschema.ProjectTodoSchema.UserID.Eq(input.UserID)).
		OrderBy(dbschema.ProjectTodoSchema.CreatedAt.Desc(), dbschema.ProjectTodoSchema.ID.Desc()).
		Limit(input.Limit)
	if state != "" {
		query = query.Where(dbschema.ProjectTodoSchema.State.Eq(state))
	}
	rows, err := dbx.QueryAll[projectTodoRow](ctx, r.db, query, r.mapper)
	if err != nil {
		return nil, persistence.NormalizeError(err)
	}
	items := collectionx.NewListWithCapacity[tododomain.ProjectTodo](rows.Len())
	rows.Range(func(_ int, row projectTodoRow) bool {
		items.Add(row.domain())
		return true
	})
	return items, nil
}

func (r *Repository) MarkDoneByIDAndUser(ctx context.Context, id, userID int64) error {
	now := time.Now().UTC()
	schema := dbschema.ProjectTodoSchema
	result, err := r.base.Update(ctx, querydsl.Update(schema).Set(
		schema.State.Set(tododomain.ProjectTodoStateDone),
		schema.DoneAt.Set(now),
		schema.UpdatedAt.Set(now),
	).Where(querydsl.And(
		schema.ID.Eq(id),
		schema.UserID.Eq(userID),
	)))
	if err != nil {
		return oops.In("persistence.todo").With("todo_id", id, "user_id", userID).Wrapf(err, "mark project todo done")
	}
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return oops.In("persistence.todo").With("todo_id", id, "user_id", userID).Wrapf(err, "read marked project todo result")
	}
	if rowsAffected == 0 {
		return todoports.ErrNotFound
	}
	return nil
}

func (r *Repository) findExisting(ctx context.Context, input CreateInput) (tododomain.ProjectTodo, error) {
	query := querydsl.Select(dbschema.ProjectTodoSchema.AllColumns().Values()...).
		From(dbschema.ProjectTodoSchema).
		Where(querydsl.And(
			dbschema.ProjectTodoSchema.UserID.Eq(input.UserID),
			dbschema.ProjectTodoSchema.Kind.Eq(strings.TrimSpace(input.Kind)),
			dbschema.ProjectTodoSchema.TargetType.Eq(strings.TrimSpace(input.TargetType)),
			dbschema.ProjectTodoSchema.TargetID.Eq(strings.TrimSpace(input.TargetID)),
		)).
		Limit(1)
	row, err := dbx.QueryOption[projectTodoRow](ctx, r.db, query, r.mapper)
	if err != nil {
		return tododomain.ProjectTodo{}, err
	}
	item, found := row.Get()
	if !found {
		return tododomain.ProjectTodo{}, todoports.ErrNotFound
	}
	return item.domain(), nil
}

func (r *Repository) wrapCreateError(err error, input CreateInput) error {
	return oops.In("persistence.todo").With("user_id", input.UserID, "project_id", input.ProjectID, "kind", input.Kind).Wrapf(err, "upsert project todo")
}

func projectTodoUpsertQuery(item tododomain.ProjectTodo) *querydsl.InsertQuery {
	schema := dbschema.ProjectTodoSchema
	return querydsl.InsertInto(schema).Values(
		schema.ID.Set(item.ID),
		schema.UserID.Set(item.UserID),
		schema.OrganizationID.Set(item.OrganizationID),
		schema.ProjectID.Set(item.ProjectID),
		schema.Kind.Set(item.Kind),
		schema.State.Set(item.State),
		schema.TargetType.Set(item.TargetType),
		schema.TargetID.Set(item.TargetID),
		schema.Title.Set(item.Title),
		schema.Summary.Set(item.Summary),
		schema.ActionURL.Set(item.ActionURL),
		schema.CreatedAt.Set(item.CreatedAt),
		schema.UpdatedAt.Set(item.UpdatedAt),
	).OnConflict(schema.UserID, schema.Kind, schema.TargetType, schema.TargetID).DoUpdateSet(
		// A no-op update makes the existing row available to RETURNING without
		// changing the original todo payload.
		schema.UserID.SetExcluded(),
	)
}

func (row projectTodoRow) domain() tododomain.ProjectTodo {
	item := tododomain.ProjectTodo{
		ID:             row.ID,
		UserID:         row.UserID,
		OrganizationID: row.OrganizationID,
		ProjectID:      row.ProjectID,
		Kind:           row.Kind,
		State:          row.State,
		TargetType:     row.TargetType,
		TargetID:       row.TargetID,
		Title:          row.Title,
		Summary:        row.Summary,
		ActionURL:      row.ActionURL,
		CreatedAt:      row.CreatedAt,
		UpdatedAt:      row.UpdatedAt,
	}
	if row.DoneAt.Valid {
		item.DoneAt = row.DoneAt.Time
	}
	return item
}
