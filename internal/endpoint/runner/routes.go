package runner

import (
	"context"
	"time"

	"github.com/DaiYuANg/gity/internal/httpapi"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	runnerservice "github.com/DaiYuANg/gity/internal/service/runner"
	"github.com/arcgolabs/httpx"
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

type runnerOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *runnerservice.Service
	projectService *projectservice.Service
	authRuntime    *platformauth.Runtime
}

func NewEndpoint(service *runnerservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Runners", "Runners", "Project runner and runner agent APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	listProjectRunners := func(ctx context.Context, in *projectRunnersInput) (*runnerOutput, error) {
		items, err := service.ListProjectRunners(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: items}, nil
	}

	registerProjectRunner := func(ctx context.Context, in *registerRunnerInput) (*runnerOutput, error) {
		item, err := service.RegisterProjectRunner(ctx, in.ProjectID, runnerservice.RegisterInput{
			Name:        in.Body.Name,
			Description: in.Body.Description,
			Tags:        in.Body.Tags,
		})
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	deleteProjectRunner := func(ctx context.Context, in *projectRunnerInput) (*runnerOutput, error) {
		item, err := service.DeleteProjectRunner(ctx, in.ProjectID, in.RunnerID)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	heartbeat := func(ctx context.Context, in *runnerTokenInput) (*runnerOutput, error) {
		item, err := service.Heartbeat(ctx, in.Body.Token)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	claimJob := func(ctx context.Context, in *claimJobInput) (*runnerOutput, error) {
		item, err := service.ClaimJob(ctx, in.Body.Token, time.Duration(in.Body.LeaseSeconds)*time.Second)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	completeJob := func(ctx context.Context, in *runnerJobInput) (*runnerOutput, error) {
		item, err := service.CompleteJob(ctx, in.Body.Token, in.JobID, in.Body.Result)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	failJob := func(ctx context.Context, in *runnerJobInput) (*runnerOutput, error) {
		item, err := service.FailJob(ctx, in.Body.Token, in.JobID, in.Body.Error, in.Body.Result, time.Duration(in.Body.RetryAfterSeconds)*time.Second)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	uploadArtifact := func(ctx context.Context, in *runnerArtifactInput) (*runnerOutput, error) {
		item, err := service.UploadArtifact(ctx, in.Body.Token, in.JobID, runnerservice.UploadArtifactInput{
			Name:          in.Body.Name,
			FileName:      in.Body.FileName,
			FilePath:      in.Body.FilePath,
			ContentType:   in.Body.ContentType,
			ContentBase64: in.Body.ContentBase64,
		})
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/runners", listProjectRunners),
		httpapi.Get("/repos/{id}/runners", listProjectRunners, httpapi.DeprecatedRoute[projectRunnersInput, runnerOutput]("Use GET /projects/{id}/runners instead.")),
		httpapi.Post("/projects/{id}/runners", registerProjectRunner, httpapi.RequireProjectWriteRoute[registerRunnerInput, runnerOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/runners", registerProjectRunner,
			httpapi.RequireProjectWriteRoute[registerRunnerInput, runnerOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[registerRunnerInput, runnerOutput]("Use POST /projects/{id}/runners instead."),
		),
		httpapi.Delete("/projects/{id}/runners/{runner_id}", deleteProjectRunner, httpapi.RequireProjectWriteRoute[projectRunnerInput, runnerOutput](authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}/runners/{runner_id}", deleteProjectRunner,
			httpapi.RequireProjectWriteRoute[projectRunnerInput, runnerOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectRunnerInput, runnerOutput]("Use DELETE /projects/{id}/runners/{runner_id} instead."),
		),
		httpapi.Post("/runners/heartbeat", heartbeat),
		httpapi.Post("/runners/jobs/claim", claimJob),
		httpapi.Post("/runners/jobs/{job_id}/complete", completeJob),
		httpapi.Post("/runners/jobs/{job_id}/fail", failJob),
		httpapi.Post("/runners/jobs/{job_id}/artifacts", uploadArtifact),
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
