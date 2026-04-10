package project

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
)

type createProjectInput struct {
	Body projectservice.CreateInput `json:"body"`
}

type projectByIDInput struct {
	ID int64 `path:"id"`
}

type listProjectsInput struct {
	NamespaceID *int64 `query:"namespace_id"`
}

type projectOutput struct {
	Body any `json:"body"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *projectservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/projects", func(ctx context.Context, in *listProjectsInput) (*projectOutput, error) {
		items, err := service.List(ctx, in.NamespaceID)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: items.Values()}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}", func(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/projects", func(ctx context.Context, in *createProjectInput) (*projectOutput, error) {
		item, err := service.Create(ctx, in.Body)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: item}, nil
	})
}
