package mergerequest

import (
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

type setParticipantsInput struct {
	ProjectID     int64            `path:"id"`
	MergeIID      int64            `path:"merge_iid"`
	Authorization string           `header:"Authorization"`
	Body          participantsBody `json:"body"`
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

type participantsBody struct {
	UserIDs []int64 `json:"user_ids"`
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
	authRuntime := e.authRuntime

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/merge-requests", e.listMergeRequests),
		httpapi.Get("/repos/{id}/merge-requests", e.listMergeRequests, httpapi.DeprecatedRoute[mergeRequestsInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}", e.getMergeRequest),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}", e.getMergeRequest, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid} instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/diff", e.getDiff),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/diff", e.getDiff, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/diff instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/checks", e.getChecks),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/checks", e.getChecks, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/checks instead.")),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/participants", e.listParticipants),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/participants", e.listParticipants, httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/participants instead.")),
		httpapi.Post("/projects/{id}/merge-requests", e.createMergeRequest, httpapi.RequireUserRoute[createMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Post("/repos/{id}/merge-requests", e.createMergeRequest,
			httpapi.RequireUserRoute[createMergeRequestInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[createMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/merge", e.mergeMergeRequest, httpapi.RequireUserRoute[mergeMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/merge", e.mergeMergeRequest,
			httpapi.RequireUserRoute[mergeMergeRequestInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[mergeMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/merge instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}/reviewers", e.setReviewers, httpapi.RequireUserRoute[setParticipantsInput, mergeRequestOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}/reviewers", e.setReviewers,
			httpapi.RequireUserRoute[setParticipantsInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[setParticipantsInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid}/reviewers instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}/assignees", e.setAssignees, httpapi.RequireUserRoute[setParticipantsInput, mergeRequestOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}/assignees", e.setAssignees,
			httpapi.RequireUserRoute[setParticipantsInput, mergeRequestOutput](authRuntime),
			httpapi.DeprecatedRoute[setParticipantsInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid}/assignees instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}", e.updateMergeRequest, httpapi.RequireUserRoute[updateMergeRequestInput, mergeRequestOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}", e.updateMergeRequest,
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

func (in setParticipantsInput) AuthorizationHeader() string {
	return in.Authorization
}
