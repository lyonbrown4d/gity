package todo

import (
	"context"

	todoservice "github.com/lyonbrown4d/gity/internal/application/todo"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

func (e *Endpoint) listTodos(ctx context.Context, in *todosInput) (*todoOutput, error) {
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, 0)
	if err != nil {
		return nil, err
	}
	items, err := e.service.List(ctx, todoservice.ListInput{UserID: userID, State: in.State, Limit: in.Limit})
	if err != nil {
		return nil, err
	}
	return &todoOutput{Body: items}, nil
}

func (e *Endpoint) markTodoDone(ctx context.Context, in *todoByIDInput) (*todoOutput, error) {
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, 0)
	if err != nil {
		return nil, err
	}
	if err := e.service.MarkDone(ctx, in.ID, userID); err != nil {
		return nil, err
	}
	return &todoOutput{Body: map[string]any{"id": in.ID, "state": "done"}}, nil
}
