package pipeline

import (
	"context"

	"github.com/DaiYuANg/gity/internal/httpapi"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	pipelineservice "github.com/DaiYuANg/gity/internal/service/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	"github.com/arcgolabs/httpx"
)

type projectPipelinesInput struct {
	ProjectID int64 `path:"id"`
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
	authRuntime    *platformauth.Runtime
}

func NewEndpoint(service *pipelineservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Pipelines", "Pipelines", "Project pipeline APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	listPipelines := func(ctx context.Context, in *projectPipelinesInput) (*pipelineOutput, error) {
		items, err := service.ListPipelines(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: items}, nil
	}

	createPipeline := func(ctx context.Context, in *createPipelineInput) (*pipelineOutput, error) {
		item, err := service.CreatePipeline(ctx, in.ProjectID, pipelineservice.CreatePipelineInput{
			Source:        in.Body.Source,
			RefName:       in.Body.RefName,
			CommitSHA:     in.Body.CommitSHA,
			ConfigSource:  in.Body.ConfigSource,
			ConfigContent: in.Body.ConfigContent,
		})
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	lintPipeline := func(ctx context.Context, in *lintPipelineInput) (*pipelineOutput, error) {
		item, err := service.LintPipeline(ctx, in.ProjectID, pipelineservice.LintInput{
			ConfigContent: in.Body.ConfigContent,
		})
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	getPipeline := func(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
		item, err := service.GetPipeline(ctx, in.ProjectID, in.PipelineID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	refreshPipeline := func(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
		item, err := service.RefreshPipeline(ctx, in.ProjectID, in.PipelineID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	cancelPipeline := func(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
		item, err := service.CancelPipeline(ctx, in.ProjectID, in.PipelineID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	retryPipeline := func(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
		item, err := service.RetryPipeline(ctx, in.ProjectID, in.PipelineID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: item}, nil
	}

	listPipelineJobs := func(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
		items, err := service.ListPipelineJobs(ctx, in.ProjectID, in.PipelineID)
		if err != nil {
			return nil, err
		}
		return &pipelineOutput{Body: items}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/pipelines", listPipelines),
		httpapi.Get("/repos/{id}/pipelines", listPipelines, httpapi.DeprecatedRoute[projectPipelinesInput, pipelineOutput]("Use GET /projects/{id}/pipelines instead.")),
		httpapi.Post("/projects/{id}/pipelines", createPipeline, httpapi.RequireProjectWriteRoute[createPipelineInput, pipelineOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/pipelines", createPipeline,
			httpapi.RequireProjectWriteRoute[createPipelineInput, pipelineOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines instead."),
		),
		httpapi.Post("/projects/{id}/ci/lint", lintPipeline, httpapi.RequireProjectWriteRoute[lintPipelineInput, pipelineOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/ci/lint", lintPipeline,
			httpapi.RequireProjectWriteRoute[lintPipelineInput, pipelineOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[lintPipelineInput, pipelineOutput]("Use POST /projects/{id}/ci/lint instead."),
		),
		httpapi.Get("/projects/{id}/pipelines/{pipeline_id}", getPipeline),
		httpapi.Get("/repos/{id}/pipelines/{pipeline_id}", getPipeline, httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use GET /projects/{id}/pipelines/{pipeline_id} instead.")),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/refresh", refreshPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/refresh", refreshPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/refresh instead."),
		),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/cancel", cancelPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/cancel", cancelPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/cancel instead."),
		),
		httpapi.Post("/projects/{id}/pipelines/{pipeline_id}/retry", retryPipeline, httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/pipelines/{pipeline_id}/retry", retryPipeline,
			httpapi.RequireProjectWriteRoute[projectPipelineInput, pipelineOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use POST /projects/{id}/pipelines/{pipeline_id}/retry instead."),
		),
		httpapi.Get("/projects/{id}/pipelines/{pipeline_id}/jobs", listPipelineJobs),
		httpapi.Get("/repos/{id}/pipelines/{pipeline_id}/jobs", listPipelineJobs, httpapi.DeprecatedRoute[projectPipelineInput, pipelineOutput]("Use GET /projects/{id}/pipelines/{pipeline_id}/jobs instead.")),
	)
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
