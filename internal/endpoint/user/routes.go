package user

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
)

type createUserInput struct {
	Body createUserBody `json:"body"`
}

type userByIDInput struct {
	ID int64 `path:"id"`
}

type userTokenInput struct {
	ID int64 `path:"id"`
}

type createUserTokenInput struct {
	ID   int64               `path:"id"`
	Body createUserTokenBody `json:"body"`
}

type userOutput struct {
	Body any `json:"body"`
}

type createUserBody struct {
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
}

type createUserTokenBody struct {
	Name string `json:"name"`
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
		item, err := service.Create(ctx, userservice.CreateInput{
			Username:    in.Body.Username,
			DisplayName: in.Body.DisplayName,
			Email:       in.Body.Email,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/users/{id}/tokens", func(ctx context.Context, in *userTokenInput) (*userOutput, error) {
		items, err := service.ListTokens(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: items}, nil
	})

	httpx.MustGroupPost(v1, "/users/{id}/tokens", func(ctx context.Context, in *createUserTokenInput) (*userOutput, error) {
		item, err := service.CreateToken(ctx, in.ID, userservice.CreateTokenInput{
			Name: in.Body.Name,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: item}, nil
	})
}
