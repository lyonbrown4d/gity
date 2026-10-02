package todo

import (
	"context"
	"errors"
	"strings"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	todoports "github.com/lyonbrown4d/gity/internal/application/ports"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
	"github.com/samber/oops"
)

type Service struct {
	repo todoports.ProjectTodoRepository
}

type CreateInput = todoports.CreateProjectTodoInput

type ListInput struct {
	UserID int64
	State  string
	Limit  int
}

func NewService(repo todoports.ProjectTodoRepository) *Service {
	return &Service{repo: repo}
}

func (s *Service) CreateIfNotExists(ctx context.Context, input CreateInput) (tododomain.ProjectTodo, error) {
	if err := validateCreateInput(input); err != nil {
		return tododomain.ProjectTodo{}, err
	}
	item, err := s.repo.CreateIfNotExists(ctx, input)
	if err != nil {
		return tododomain.ProjectTodo{}, oops.In("todo").With("user_id", input.UserID, "project_id", input.ProjectID, "kind", input.Kind).Wrapf(err, "create project todo")
	}
	return item, nil
}

func (s *Service) List(ctx context.Context, input ListInput) ([]tododomain.ProjectTodo, error) {
	if input.UserID <= 0 {
		return nil, oops.In("todo").With("user_id", input.UserID).New("todo user id is required")
	}
	state := strings.TrimSpace(input.State)
	if state != "" && state != tododomain.ProjectTodoStatePending && state != tododomain.ProjectTodoStateDone {
		return nil, apperror.BadRequest("unsupported todo state", oops.In("todo").With("state", state).New("unsupported todo state"))
	}
	items, err := s.repo.ListByUserID(ctx, todoports.ListProjectTodosInput{UserID: input.UserID, State: state, Limit: input.Limit})
	if err != nil {
		return nil, oops.In("todo").With("user_id", input.UserID, "state", state).Wrapf(err, "list project todos")
	}
	return items.Values(), nil
}

func (s *Service) MarkDone(ctx context.Context, id, userID int64) error {
	if id <= 0 {
		return apperror.BadRequest("todo id is required", oops.In("todo").With("todo_id", id).New("todo id is required"))
	}
	if userID <= 0 {
		return oops.In("todo").With("user_id", userID).New("todo user id is required")
	}
	if err := s.repo.MarkDoneByIDAndUser(ctx, id, userID); err != nil {
		if errors.Is(err, todoports.ErrNotFound) {
			return apperror.NotFound("todo not found", err)
		}
		return oops.In("todo").With("todo_id", id, "user_id", userID).Wrapf(err, "mark project todo done")
	}
	return nil
}

func validateCreateInput(input CreateInput) error {
	if input.UserID <= 0 {
		return oops.In("todo").With("user_id", input.UserID).New("todo user id is required")
	}
	if input.ProjectID <= 0 {
		return oops.In("todo").With("project_id", input.ProjectID).New("todo project id is required")
	}
	if strings.TrimSpace(input.Kind) == "" {
		return oops.In("todo").With("project_id", input.ProjectID).New("todo kind is required")
	}
	if strings.TrimSpace(input.TargetType) == "" || strings.TrimSpace(input.TargetID) == "" {
		return oops.In("todo").With("project_id", input.ProjectID, "kind", input.Kind).New("todo target is required")
	}
	if strings.TrimSpace(input.Title) == "" {
		return oops.In("todo").With("project_id", input.ProjectID, "kind", input.Kind).New("todo title is required")
	}
	return nil
}
