package namespace

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
)

type createNamespaceInput struct {
	Body namespaceservice.CreateInput `json:"body"`
}

type namespaceByIDInput struct {
	ID int64 `path:"id"`
}

type namespaceOutput struct {
	Body any `json:"body"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *namespaceservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/namespaces", func(ctx context.Context, in *struct{}) (*namespaceOutput, error) {
		_ = in
		items, err := service.List(ctx)
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: items.Values()}, nil
	})

	httpx.MustGroupGet(v1, "/namespaces/{id}", func(ctx context.Context, in *namespaceByIDInput) (*namespaceOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/namespaces", func(ctx context.Context, in *createNamespaceInput) (*namespaceOutput, error) {
		item, err := service.Create(ctx, in.Body)
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: item}, nil
	})
}
