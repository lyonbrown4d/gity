package projecttodo

import (
	"context"
	"errors"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	todoports "github.com/lyonbrown4d/gity/internal/application/ports"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[tododomain.ProjectTodo, dbschema.ProjectTodoSchemaDef]
}

type CreateInput = todoports.CreateProjectTodoInput
type ListInput = todoports.ListProjectTodosInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[tododomain.ProjectTodo](db, dbschema.ProjectTodoSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectTodoRepository(repo *Repository) todoports.ProjectTodoRepository {
	return repo
}

func (r *Repository) CreateIfNotExists(ctx context.Context, input CreateInput) (tododomain.ProjectTodo, error) {
	existing, err := r.findExisting(ctx, input)
	if err == nil {
		return existing, nil
	}
	if !errors.Is(err, todoports.ErrNotFound) {
		return tododomain.ProjectTodo{}, oops.In("persistence.todo").With("user_id", input.UserID, "project_id", input.ProjectID, "kind", input.Kind).Wrapf(err, "find existing project todo")
	}

	now := time.Now().UTC()
	item := tododomain.ProjectTodo{
		UserID:         input.UserID,
		OrganizationID: input.OrganizationID,
		ProjectID:      input.ProjectID,
		Kind:           strings.TrimSpace(input.Kind),
		State:          tododomain.ProjectTodoStatePending,
		TargetType:     strings.TrimSpace(input.TargetType),
		TargetID:       strings.TrimSpace(input.TargetID),
		Title:          strings.TrimSpace(input.Title),
		Summary:        strings.TrimSpace(input.Summary),
		ActionURL:      strings.TrimSpace(input.ActionURL),
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return tododomain.ProjectTodo{}, oops.In("persistence.todo").With("user_id", input.UserID, "project_id", input.ProjectID, "kind", input.Kind).Wrapf(err, "insert project todo")
	}
	return item, nil
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
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) MarkDoneByIDAndUser(ctx context.Context, id, userID int64) error {
	item, err := persistence.One(r.base.By(dbschema.ProjectTodoSchema.ID).Get(ctx, id))
	if err != nil {
		return oops.In("persistence.todo").With("todo_id", id, "user_id", userID).Wrapf(err, "load project todo")
	}
	if item.UserID != userID {
		return todoports.ErrNotFound
	}
	now := time.Now().UTC()
	if _, err := dbxrepo.PatchSet(r.base, projectTodoKey(id)).Set(
		dbschema.ProjectTodoSchema.State.Set(tododomain.ProjectTodoStateDone),
		dbschema.ProjectTodoSchema.DoneAt.Set(now),
		dbschema.ProjectTodoSchema.UpdatedAt.Set(now),
	).Apply(ctx); err != nil {
		return oops.In("persistence.todo").With("todo_id", id, "user_id", userID).Wrapf(err, "mark project todo done")
	}
	return nil
}

func (r *Repository) findExisting(ctx context.Context, input CreateInput) (tododomain.ProjectTodo, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectTodoSchema.UserID.Eq(input.UserID)).
		Where(dbschema.ProjectTodoSchema.Kind.Eq(strings.TrimSpace(input.Kind))).
		Where(dbschema.ProjectTodoSchema.TargetType.Eq(strings.TrimSpace(input.TargetType))).
		Where(dbschema.ProjectTodoSchema.TargetID.Eq(strings.TrimSpace(input.TargetID))).
		First(ctx))
}

func projectTodoKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectTodoSchema.ID, id))
}
