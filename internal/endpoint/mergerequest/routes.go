package mergerequest

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	mergerequestservice "github.com/DaiYuANg/gity/internal/service/mergerequest"
)

type mergeRequestInput struct {
	ProjectID int64 `path:"id"`
	MergeIID  int64 `path:"merge_iid"`
}

type createMergeRequestInput struct {
	ProjectID int64                           `path:"id"`
	Body      mergerequestservice.CreateInput `json:"body"`
}

type updateMergeRequestInput struct {
	ProjectID int64                           `path:"id"`
	MergeIID  int64                           `path:"merge_iid"`
	Body      mergerequestservice.UpdateInput `json:"body"`
}

type mergeRequestOutput struct {
	Body any `json:"body"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *mergerequestservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/projects/{id}/merge-requests", func(ctx context.Context, in *struct {
		ProjectID int64 `path:"id"`
	}) (*mergeRequestOutput, error) { items, err := service.List(ctx, in.ProjectID); if err != nil {
		return nil, err
	}; return &mergeRequestOutput{Body: items}, nil })

	httpx.MustGroupGet(v1, "/projects/{id}/merge-requests/{merge_iid}", func(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.GetByIID(ctx, in.ProjectID, in.MergeIID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/merge-requests", func(ctx context.Context, in *createMergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.Create(ctx, in.ProjectID, in.Body)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})

	httpx.MustGroupPatch(v1, "/projects/{id}/merge-requests/{merge_iid}", func(ctx context.Context, in *updateMergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.Update(ctx, in.ProjectID, in.MergeIID, in.Body)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})
}
