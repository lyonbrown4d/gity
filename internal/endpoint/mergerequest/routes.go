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
	ProjectID int64                  `path:"id"`
	Body      createMergeRequestBody `json:"body"`
}

type updateMergeRequestInput struct {
	ProjectID int64                  `path:"id"`
	MergeIID  int64                  `path:"merge_iid"`
	Body      updateMergeRequestBody `json:"body"`
}

type mergeRequestOutput struct {
	Body any `json:"body"`
}

type createMergeRequestBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
	SourceBranch string `json:"source_branch"`
	TargetBranch string `json:"target_branch"`
}

type updateMergeRequestBody struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *mergerequestservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/projects/{id}/merge-requests", func(ctx context.Context, in *struct {
		ProjectID int64 `path:"id"`
	}) (*mergeRequestOutput, error) {
		items, err := service.List(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/merge-requests/{merge_iid}", func(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.GetByIID(ctx, in.ProjectID, in.MergeIID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/merge-requests", func(ctx context.Context, in *createMergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.Create(ctx, in.ProjectID, mergerequestservice.CreateInput{
			AuthorUserID: in.Body.AuthorUserID,
			Title:        in.Body.Title,
			Description:  in.Body.Description,
			SourceBranch: in.Body.SourceBranch,
			TargetBranch: in.Body.TargetBranch,
		})
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})

	httpx.MustGroupPatch(v1, "/projects/{id}/merge-requests/{merge_iid}", func(ctx context.Context, in *updateMergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.Update(ctx, in.ProjectID, in.MergeIID, mergerequestservice.UpdateInput{
			Title:       in.Body.Title,
			Description: in.Body.Description,
			State:       in.Body.State,
		})
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	})
}
