package runner

import (
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
)

type projectRunnersInput struct {
	ProjectID int64 `path:"id"`
}

type projectRunnerInput struct {
	ProjectID     int64  `path:"id"`
	RunnerID      int64  `path:"runner_id"`
	Authorization string `header:"Authorization"`
}

type registerRunnerInput struct {
	ProjectID     int64              `path:"id"`
	Authorization string             `header:"Authorization"`
	Body          registerRunnerBody `json:"body"`
}

type registerRunnerBody struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Tags        string `json:"tags"`
}

type runnerTokenInput struct {
	Body runnerTokenBody `json:"body"`
}

type claimJobInput struct {
	Body claimJobBody `json:"body"`
}

type runnerJobInput struct {
	JobID int64         `path:"job_id"`
	Body  runnerJobBody `json:"body"`
}

type runnerTokenBody struct {
	Token string `json:"token"`
}

type runnerSourceArchiveInput struct {
	JobID int64           `path:"job_id"`
	Body  runnerTokenBody `json:"body"`
}

type claimJobBody struct {
	Token        string `json:"token"`
	LeaseSeconds int    `json:"lease_seconds"`
}

type runnerJobBody struct {
	Token             string `json:"token"`
	Result            string `json:"result"`
	Error             string `json:"error"`
	RetryAfterSeconds int    `json:"retry_after_seconds"`
}

type runnerTraceBody struct {
	Token           string `json:"token"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
}

type runnerArtifactBody struct {
	Token         string `json:"token"`
	Name          string `json:"name"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

type runnerArtifactInput struct {
	JobID int64              `path:"job_id"`
	Body  runnerArtifactBody `json:"body"`
}

type runnerTraceInput struct {
	JobID int64           `path:"job_id"`
	Body  runnerTraceBody `json:"body"`
}

type runnerOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *runnerservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *runnerservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Runners", "Runners", "Project runner and runner agent APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/runners", e.listProjectRunners),
		httpapi.Get("/repos/{id}/runners", e.listProjectRunners, httpapi.DeprecatedRoute[projectRunnersInput, runnerOutput]("Use GET /projects/{id}/runners instead.")),
		httpapi.Post("/projects/{id}/runners", e.registerProjectRunner, httpapi.RequireProjectWriteRoute[registerRunnerInput, runnerOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/runners", e.registerProjectRunner,
			httpapi.RequireProjectWriteRoute[registerRunnerInput, runnerOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[registerRunnerInput, runnerOutput]("Use POST /projects/{id}/runners instead."),
		),
		httpapi.Delete("/projects/{id}/runners/{runner_id}", e.deleteProjectRunner, httpapi.RequireProjectWriteRoute[projectRunnerInput, runnerOutput](authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}/runners/{runner_id}", e.deleteProjectRunner,
			httpapi.RequireProjectWriteRoute[projectRunnerInput, runnerOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectRunnerInput, runnerOutput]("Use DELETE /projects/{id}/runners/{runner_id} instead."),
		),
		httpapi.Post("/runners/heartbeat", e.heartbeat),
		httpapi.Post("/runners/jobs/claim", e.claimJob),
		httpapi.Post("/runners/jobs/{job_id}/complete", e.completeJob),
		httpapi.Post("/runners/jobs/{job_id}/fail", e.failJob),
		httpapi.Post("/runners/jobs/{job_id}/trace", e.appendTrace),
		httpapi.Post("/runners/jobs/{job_id}/source-archive", e.downloadSourceArchive),
		httpapi.Post("/runners/jobs/{job_id}/artifacts", e.uploadArtifact),
	)
}

func (in projectRunnerInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectRunnerInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in registerRunnerInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in registerRunnerInput) ProjectIDValue() int64 {
	return in.ProjectID
}
