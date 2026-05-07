package job

import (
	"context"
	"time"

	"github.com/DaiYuANg/gity/internal/httpapi"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	jobservice "github.com/DaiYuANg/gity/internal/service/job"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	"github.com/arcgolabs/httpx"
)

type projectJobsInput struct {
	ProjectID int64 `path:"id"`
}

type projectJobInput struct {
	ProjectID     int64  `path:"id"`
	JobID         int64  `path:"job_id"`
	Authorization string `header:"Authorization"`
}

type projectJobArtifactInput struct {
	ProjectID  int64 `path:"id"`
	JobID      int64 `path:"job_id"`
	ArtifactID int64 `path:"artifact_id"`
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
	service        *jobservice.Service
	projectService *projectservice.Service
	authRuntime    *platformauth.Runtime
}

func NewEndpoint(service *jobservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Jobs", "Jobs", "Project job APIs.")
}

func RegisterRoutes(server httpx.ServerRuntime, service *jobservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) {
	server.RegisterOnly(NewEndpoint(service, projectService, authRuntime))
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	listProjectJobs := func(ctx context.Context, in *projectJobsInput) (*jobOutput, error) {
		items, err := service.ListProjectJobs(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: items}, nil
	}

	createJob := func(ctx context.Context, in *createJobInput) (*jobOutput, error) {
		runAfter, err := parseRunAfter(in.Body.RunAfter)
		if err != nil {
			return nil, err
		}
		item, err := service.EnqueueProjectJob(ctx, in.ProjectID, jobservice.CreateInput{
			Kind:        in.Body.Kind,
			Payload:     in.Body.Payload,
			MaxAttempts: in.Body.MaxAttempts,
			RunAfter:    runAfter,
		})
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	getProjectJob := func(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
		item, err := service.GetProjectJob(ctx, in.ProjectID, in.JobID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	cancelProjectJob := func(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
		item, err := service.CancelProjectJob(ctx, in.ProjectID, in.JobID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	retryProjectJob := func(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
		item, err := service.RetryProjectJob(ctx, in.ProjectID, in.JobID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	getProjectJobTrace := func(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
		item, err := service.GetProjectJobTrace(ctx, in.ProjectID, in.JobID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	listProjectJobArtifacts := func(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
		items, err := service.ListProjectJobArtifacts(ctx, in.ProjectID, in.JobID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: items}, nil
	}

	getProjectJobArtifact := func(ctx context.Context, in *projectJobArtifactInput) (*jobOutput, error) {
		item, err := service.GetProjectJobArtifactContent(ctx, in.ProjectID, in.JobID, in.ArtifactID)
		if err != nil {
			return nil, err
		}
		return &jobOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/jobs", listProjectJobs),
		httpapi.Get("/repos/{id}/jobs", listProjectJobs, httpapi.DeprecatedRoute[projectJobsInput, jobOutput]("Use GET /projects/{id}/jobs instead.")),
		httpapi.Post("/projects/{id}/jobs", createJob, httpapi.RequireProjectWriteRoute[createJobInput, jobOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/jobs", createJob,
			httpapi.RequireProjectWriteRoute[createJobInput, jobOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createJobInput, jobOutput]("Use POST /projects/{id}/jobs instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}", getProjectJob),
		httpapi.Get("/repos/{id}/jobs/{job_id}", getProjectJob, httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id} instead.")),
		httpapi.Post("/projects/{id}/jobs/{job_id}/cancel", cancelProjectJob, httpapi.RequireProjectWriteRoute[projectJobInput, jobOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/jobs/{job_id}/cancel", cancelProjectJob,
			httpapi.RequireProjectWriteRoute[projectJobInput, jobOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use POST /projects/{id}/jobs/{job_id}/cancel instead."),
		),
		httpapi.Post("/projects/{id}/jobs/{job_id}/retry", retryProjectJob, httpapi.RequireProjectWriteRoute[projectJobInput, jobOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/jobs/{job_id}/retry", retryProjectJob,
			httpapi.RequireProjectWriteRoute[projectJobInput, jobOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use POST /projects/{id}/jobs/{job_id}/retry instead."),
		),
		httpapi.Get("/projects/{id}/jobs/{job_id}/trace", getProjectJobTrace),
		httpapi.Get("/repos/{id}/jobs/{job_id}/trace", getProjectJobTrace, httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/trace instead.")),
		httpapi.Get("/projects/{id}/jobs/{job_id}/artifacts", listProjectJobArtifacts),
		httpapi.Get("/repos/{id}/jobs/{job_id}/artifacts", listProjectJobArtifacts, httpapi.DeprecatedRoute[projectJobInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/artifacts instead.")),
		httpapi.Get("/projects/{id}/jobs/{job_id}/artifacts/{artifact_id}", getProjectJobArtifact),
		httpapi.Get("/repos/{id}/jobs/{job_id}/artifacts/{artifact_id}", getProjectJobArtifact, httpapi.DeprecatedRoute[projectJobArtifactInput, jobOutput]("Use GET /projects/{id}/jobs/{job_id}/artifacts/{artifact_id} instead.")),
	)
}

func (in projectJobInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectJobInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createJobInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createJobInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func parseRunAfter(value string) (time.Time, error) {
	if value == "" {
		return time.Time{}, nil
	}
	return time.Parse(time.RFC3339, value)
}
