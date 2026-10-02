package ports

import (
	"context"

	collectionx "github.com/arcgolabs/collectionx/list"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
)

type ProjectTodoRepository interface {
	CreateIfNotExists(ctx context.Context, input CreateProjectTodoInput) (tododomain.ProjectTodo, error)
	ListByUserID(ctx context.Context, input ListProjectTodosInput) (*collectionx.List[tododomain.ProjectTodo], error)
	MarkDoneByIDAndUser(ctx context.Context, id, userID int64) error
}

type CreateProjectTodoInput struct {
	UserID         int64
	OrganizationID int64
	ProjectID      int64
	Kind           string
	TargetType     string
	TargetID       string
	Title          string
	Summary        string
	ActionURL      string
}

type ListProjectTodosInput struct {
	UserID int64
	State  string
	Limit  int
}
