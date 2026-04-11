package user

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
)

type createUserInput struct {
	Body userservice.CreateInput `json:"body"`
}

type userByIDInput struct {
	ID int64 `path:"id"`
}

type userOutput struct {
	Body any `json:"body"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *userservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/users", func(ctx context.Context, in *struct{}) (*userOutput, error) {
		_ = in
		items, err := service.List(ctx)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: items.Values()}, nil
	})

	httpx.MustGroupGet(v1, "/users/{id}", func(ctx context.Context, in *userByIDInput) (*userOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/users", func(ctx context.Context, in *createUserInput) (*userOutput, error) {
		item, err := service.Create(ctx, in.Body)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: item}, nil
	})
}
