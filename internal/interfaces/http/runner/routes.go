package runner

import (
	"context"
	"time"

	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/httpapi"
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
		input, err := mapperx.MapStrict[runnerservice.RegisterInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.RegisterProjectRunner(ctx, in.ProjectID, input)
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

	appendTrace := func(ctx context.Context, in *runnerTraceInput) (*runnerOutput, error) {
		input, err := mapperx.MapStrict[runnerservice.AppendTraceInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.AppendTrace(ctx, in.Body.Token, in.JobID, input)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	downloadSourceArchive := func(ctx context.Context, in *runnerSourceArchiveInput) (*runnerOutput, error) {
		item, err := service.DownloadSourceArchive(ctx, in.Body.Token, in.JobID)
		if err != nil {
			return nil, err
		}
		return &runnerOutput{Body: item}, nil
	}

	uploadArtifact := func(ctx context.Context, in *runnerArtifactInput) (*runnerOutput, error) {
		input, err := mapperx.MapStrict[runnerservice.UploadArtifactInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.UploadArtifact(ctx, in.Body.Token, in.JobID, input)
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
		httpapi.Post("/runners/jobs/{job_id}/trace", appendTrace),
		httpapi.Post("/runners/jobs/{job_id}/source-archive", downloadSourceArchive),
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
