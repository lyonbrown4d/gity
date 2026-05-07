package mergerequest

import (
	"context"

	mergerequestservice "github.com/DaiYuANg/gity/internal/application/merge_request"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
)

type mergeRequestInput struct {
	ProjectID int64 `path:"id"`
	MergeIID  int64 `path:"merge_iid"`
}

type mergeRequestsInput struct {
	ProjectID int64 `path:"id"`
}

type createMergeRequestInput struct {
	ProjectID     int64                  `path:"id"`
	Authorization string                 `header:"Authorization"`
	Body          createMergeRequestBody `json:"body"`
}

type updateMergeRequestInput struct {
	ProjectID     int64                  `path:"id"`
	MergeIID      int64                  `path:"merge_iid"`
	Authorization string                 `header:"Authorization"`
	Body          updateMergeRequestBody `json:"body"`
}

type mergeMergeRequestInput struct {
	ProjectID     int64                 `path:"id"`
	MergeIID      int64                 `path:"merge_iid"`
	Authorization string                `header:"Authorization"`
	Body          mergeMergeRequestBody `json:"body"`
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

type mergeMergeRequestBody struct {
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
}

type Endpoint struct {
	service     *mergerequestservice.Service
	authRuntime *infraauth.Runtime
	mapper      *mapper.Mapper
}

func NewEndpoint(service *mergerequestservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Merge Requests", "Merge Requests", "Project merge request APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime

	listMergeRequests := func(ctx context.Context, in *mergeRequestsInput) (*mergeRequestOutput, error) {
		items, err := service.List(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: items}, nil
	}

	getMergeRequest := func(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.GetByIID(ctx, in.ProjectID, in.MergeIID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}

	getDiff := func(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.GetDiff(ctx, in.ProjectID, in.MergeIID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}

	getChecks := func(ctx context.Context, in *mergeRequestInput) (*mergeRequestOutput, error) {
		item, err := service.GetChecks(ctx, in.ProjectID, in.MergeIID)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}

	createMergeRequest := func(ctx context.Context, in *createMergeRequestInput) (*mergeRequestOutput, error) {
		input, err := mapperx.MapStrict[mergerequestservice.CreateInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		authorUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.AuthorUserID)
		if err != nil {
			return nil, err
		}
		input.AuthorUserID = authorUserID
		item, err := service.Create(ctx, in.ProjectID, input)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}

	mergeMergeRequest := func(ctx context.Context, in *mergeMergeRequestInput) (*mergeRequestOutput, error) {
		input, err := mapperx.MapStrict[mergerequestservice.MergeInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.Merge(ctx, in.ProjectID, in.MergeIID, input)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}

	updateMergeRequest := func(ctx context.Context, in *updateMergeRequestInput) (*mergeRequestOutput, error) {
		input, err := mapperx.MapStrict[mergerequestservice.UpdateInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.Update(ctx, in.ProjectID, in.MergeIID, input)
		if err != nil {
			return nil, err
		}
		return &mergeRequestOutput{Body: item}, nil
	}
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/merge-requests", listMergeRequests),
		httpapi.Get("/repos/{id}/merge-requests", listMergeRequests, httpapi.DeprecatedRoute[mergeRequestsInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}", getMergeRequest),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}", getMergeRequest, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid} instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/diff", getDiff),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/diff", getDiff, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/diff instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/checks", getChecks),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/checks", getChecks, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/checks instead.")),
		httpapi.Post("/projects/{id}/merge-requests", createMergeRequest, httpapi.RequireUserRoute[createMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Post("/repos/{id}/merge-requests", createMergeRequest,
			httpapi.RequireUserRoute[createMergeRequestInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[createMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/merge", mergeMergeRequest, httpapi.RequireUserRoute[mergeMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/merge", mergeMergeRequest,
			httpapi.RequireUserRoute[mergeMergeRequestInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[mergeMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/merge instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}", updateMergeRequest, httpapi.RequireUserRoute[updateMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}", updateMergeRequest,
			httpapi.RequireUserRoute[updateMergeRequestInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[updateMergeRequestInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid} instead."),
		),
	)
}

func (in createMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}
