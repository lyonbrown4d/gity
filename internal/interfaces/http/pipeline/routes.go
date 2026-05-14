package pipeline

import (
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type projectPipelinesInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type projectPipelineInput struct {
	ProjectID     int64  `path:"id"`
	PipelineID    int64  `path:"pipeline_id"`
	Authorization string `header:"Authorization"`
}

type createPipelineInput struct {
	ProjectID     int64              `path:"id"`
	Authorization string             `header:"Authorization"`
	Body          createPipelineBody `json:"body"`
}

type lintPipelineInput struct {
	ProjectID     int64            `path:"id"`
	Authorization string           `header:"Authorization"`
	Body          lintPipelineBody `json:"body"`
}

type createPipelineBody struct {
	Source        string `json:"source"`
	RefName       string `json:"ref_name"`
	CommitSHA     string `json:"commit_sha"`
	ConfigSource  string `json:"config_source"`
	ConfigContent string `json:"config_content"`
}

type lintPipelineBody struct {
	ConfigContent string `json:"config_content"`
}

type pipelineOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *pipelineservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *pipelineservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Pipelines", "Pipelines", "Project pipeline APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)
	pipelineRead := httpapi.RequireProjectActionRoute[projectPipelinesInput, pipelineOutput]("require_pipeline_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)
	pipelineDetailRead := httpapi.RequireProjectActionRoute[projectPipelineInput, pipelineOutput]("require_pipeline_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/pipelines", e.listPipelines, pipelineRead),
		httpapi.Get("/repos/{id}/pipelines", e.listPipelines, pipelineRead, httpapi.DeprecatedRoute[projectPipelinesInput, pipelineOutput]("Use GET /projects/{id}/pipelines instead.")),
		httpapi.Post("/projects/{id}/pipelines", e.createPipeline, httpapi.RequireProjectWriteRoute[createPipelineInput, pipelineOutput](authRuntime, projectScope)),
		httpapi.Post("/repos/{id}/pipelines", e.createPipeline,
			httpapi.RequireProjectWriteRoute[createPipelineInput, pipelineOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[createPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines instead."),
		),
		httpapi.Post("/projects/{id}/ci/lint", e.lintPipeline, httpapi.RequireProjectWriteRoute[lintPipelineInput, pipelineOutput](authRuntime, projectScope)),
		httpapi.Post("/repos/{id}/ci/lint", e.lintPipeline,
			httpapi.RequireProjectWriteRoute[lintPipelineInput, pipelineOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[lintPipelineInput, pipelineOutput]("Use POST /projects/{id}/ci/lint instead."),
		),
		httpapi.Get("/projects/{id}/pipelines/{pipeline_id}", e.getPipeline, pipelineDetailRead),
		httpapi.Get("/repos/{id}/pipelines/{pipeline_id}", e.getPipeline, pipelineDetailRead, httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use GET /projects/{id}/pipelines/{pipeline_id} instead.")),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/refresh", e.refreshPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/refresh", e.refreshPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/refresh instead."),
		),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/cancel", e.cancelPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/cancel", e.cancelPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/cancel instead."),
		),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/retry", e.retryPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/retry", e.retryPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/retry instead."),
		),
		httpapi.Get("/projects/{id}/pipelines/{pipeline_id}/jobs", e.listPipelineJobs, pipelineDetailRead),
		httpapi.Get("/repos/{id}/pipelines/{pipeline_id}/jobs", e.listPipelineJobs, pipelineDetailRead, httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use GET /projects/{id}/pipelines/{pipeline_id}/jobs instead.")),
	)
}

func (in projectPipelinesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectPipelinesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectPipelineInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectPipelineInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createPipelineInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createPipelineInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in lintPipelineInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in lintPipelineInput) ProjectIDValue() int64 {
	return in.ProjectID
}
