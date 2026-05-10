package job

import (
	"context"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
)

type projectJobsInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type projectJobInput struct {
	ProjectID     int64  `path:"id"`
	JobID         int64  `path:"job_id"`
	Authorization string `header:"Authorization"`
}

type projectJobTraceInput struct {
	ProjectID     int64  `path:"id"`
	JobID         int64  `path:"job_id"`
	Authorization string `header:"Authorization"`
	Offset        int    `query:"offset"`
	Limit         int    `query:"limit"`
}

type projectJobArtifactInput struct {
	ProjectID     int64  `path:"id"`
	JobID         int64  `path:"job_id"`
	ArtifactID    int64  `path:"artifact_id"`
	Authorization string `header:"Authorization"`
}

type createJobInput struct {
	ProjectID     int64         `path:"id"`
	Authorization string        `header:"Authorization"`
	Body          createJobBody `json:"body"`
}

type createJobBody struct {
	Kind        string `json:"kind"`
	Payload     string `json:"payload"`
	MaxAttempts int    `json:"max_attempts"`
	RunAfter    string `json:"run_after"`
}

type jobOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service         *jobservice.Service
	projectService  *projectservice.Service
	authRuntime     *infraauth.Runtime
	pipelineService *pipelineservice.Service
	mapper          *mapper.Mapper
}

func NewEndpoint(service *jobservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, pipelineService *pipelineservice.Service, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, pipelineService: pipelineService, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Jobs", "Jobs", "Project job APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/jobs", e.listProjectJobs, httpapi.RequireProjectActionRoute[projectJobsInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)),
		httpapi.Get("/repos/{id}/jobs", e.listProjectJobs,
			httpapi.RequireProjectActionRoute[projectJobsInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead),
			httpapi.DeprecatedRoute[projectJobsInput, jobOutput]("Use GET /projects/{id}/jobs instead."),
		),
		httpapi.Post("/projects/{id}/jobs", e.createJob, httpapi.RequireProjectActionRoute[createJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite)),
		httpapi.Post("/repos/{id}/jobs", e.createJob,
			httpapi.RequireProjectActionRoute[createJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite),
			httpapi.DeprecatedRoute[createJobInput, jobOutput]("Use POST /projects/{id}/jobs instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}", e.getProjectJob, httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)),
		httpapi.Get("/repos/{id}/jobs/{job_id}", e.getProjectJob,
			httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id} instead."),
		),
		httpapi.Post("/projects/{id}/jobs/{job_id}/cancel", e.cancelProjectJob, httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite)),
		httpapi.Post("/repos/{id}/jobs/{job_id}/cancel", e.cancelProjectJob,
			httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use POST /projects/{id}/jobs/{job_id}/cancel instead."),
		),
		httpapi.Post("/projects/{id}/jobs/{job_id}/retry", e.retryProjectJob, httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite)),
		httpapi.Post("/repos/{id}/jobs/{job_id}/retry", e.retryProjectJob,
			httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_write", authRuntime, projectScope, infraauth.ProjectActionJobWrite),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use POST /projects/{id}/jobs/{job_id}/retry instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}/trace", e.getProjectJobTrace, httpapi.RequireProjectActionRoute[projectJobTraceInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)),
		httpapi.Get("/repos/{id}/jobs/{job_id}/trace", e.getProjectJobTrace,
			httpapi.RequireProjectActionRoute[projectJobTraceInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead),
			httpapi.DeprecatedRoute[projectJobTraceInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/trace instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}/artifacts", e.listProjectJobArtifacts, httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)),
		httpapi.Get("/repos/{id}/jobs/{job_id}/artifacts", e.listProjectJobArtifacts,
			httpapi.RequireProjectActionRoute[projectJobInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/artifacts instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}/artifacts/{artifact_id}", e.getProjectJobArtifact, httpapi.RequireProjectActionRoute[projectJobArtifactInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead)),
		httpapi.Get("/repos/{id}/jobs/{job_id}/artifacts/{artifact_id}", e.getProjectJobArtifact,
			httpapi.RequireProjectActionRoute[projectJobArtifactInput, jobOutput]("require_job_read", authRuntime, projectScope, infraauth.ProjectActionJobRead),
			httpapi.DeprecatedRoute[projectJobArtifactInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/artifacts/{artifact_id} instead."),
		),
	)
}

func (in projectJobsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectJobsInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectJobInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectJobInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectJobTraceInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectJobTraceInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectJobArtifactInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectJobArtifactInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createJobInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createJobInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (e *Endpoint) refreshPipelineForJob(ctx context.Context, projectID, jobID int64) error {
	if e.pipelineService == nil {
		return nil
	}
	return e.pipelineService.RefreshProjectJob(ctx, projectID, jobID)
}
