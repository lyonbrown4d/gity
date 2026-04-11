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

type projectRepositoryInput struct {
	ID    int64  `path:"id"`
	Ref   string `query:"ref"`
	Path  string `query:"path"`
	Limit int    `query:"limit"`
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

	httpx.MustGroupGet(v1, "/projects/{id}/repository/branches", func(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
		items, err := service.ListBranches(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/repository/commits", func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		items, err := service.ListCommits(ctx, in.ID, in.Ref, in.Limit)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/repository/tree", func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		items, err := service.ListTree(ctx, in.ID, in.Ref, in.Path)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/repository/blob", func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		item, err := service.GetBlob(ctx, in.ID, in.Ref, in.Path)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/repository/readme", func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		item, err := service.GetReadme(ctx, in.ID, in.Ref)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: item}, nil
	})
}
