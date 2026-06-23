package runner

import (
	"context"
	"errors"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

type Service struct {
	projectRepo  gitports.ProjectRepository
	runnerRepo   gitports.ProjectRunnerRepository
	variableRepo gitports.ProjectCIVariableRepository
	jobService   *jobservice.Service
	pipelineSvc  *pipelineservice.Service
	gitRunner    gitports.GitRunner
}

type Dependencies struct {
	ProjectRepo  gitports.ProjectRepository
	RunnerRepo   gitports.ProjectRunnerRepository
	VariableRepo gitports.ProjectCIVariableRepository
	JobService   *jobservice.Service
	PipelineSvc  *pipelineservice.Service
	GitRunner    gitports.GitRunner
}

func NewDependencies(projectRepo gitports.ProjectRepository, runnerRepo gitports.ProjectRunnerRepository, variableRepo gitports.ProjectCIVariableRepository, jobService *jobservice.Service, pipelineSvc *pipelineservice.Service, gitRunner gitports.GitRunner) Dependencies {
	return Dependencies{ProjectRepo: projectRepo, RunnerRepo: runnerRepo, VariableRepo: variableRepo, JobService: jobService, PipelineSvc: pipelineSvc, GitRunner: gitRunner}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{projectRepo: dependencies.ProjectRepo, runnerRepo: dependencies.RunnerRepo, variableRepo: dependencies.VariableRepo, jobService: dependencies.JobService, pipelineSvc: dependencies.PipelineSvc, gitRunner: dependencies.GitRunner}
}

func NewService(projectRepo gitports.ProjectRepository, runnerRepo gitports.ProjectRunnerRepository, variableRepo gitports.ProjectCIVariableRepository, jobService *jobservice.Service, pipelineSvc *pipelineservice.Service, gitRunner gitports.GitRunner) *Service {
	return NewServiceWithDependencies(NewDependencies(projectRepo, runnerRepo, variableRepo, jobService, pipelineSvc, gitRunner))
}

func (s *Service) ListProjectRunners(ctx context.Context, projectID int64) ([]RunnerView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.runnerRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("runner").With("project_id", projectID).Wrapf(err, "list project runners")
	}
	return collectionlist.MapList(items, func(_ int, item cidomain.ProjectRunner) RunnerView {
		return toRunnerView(item)
	}).Values(), nil
}

func (s *Service) RegisterProjectRunner(ctx context.Context, projectID int64, input RegisterInput) (RegistrationView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return RegistrationView{}, apperror.NotFound("project not found", err)
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		return RegistrationView{}, apperror.BadRequest("runner name is required", oops.In("runner").With("project_id", projectID).New("runner name is required"))
	}
	token, err := generateRunnerToken()
	if err != nil {
		return RegistrationView{}, oops.In("runner").With("project_id", projectID).Wrapf(err, "generate runner token")
	}
	item, err := s.runnerRepo.Create(ctx, gitports.CreateProjectRunnerInput{
		ProjectID:   projectID,
		Name:        name,
		Description: input.Description,
		Tags:        input.Tags,
		Token:       token,
	})
	if err != nil {
		return RegistrationView{}, oops.In("runner").With("project_id", projectID, "name", name).Wrapf(err, "create project runner")
	}
	return RegistrationView{Runner: toRunnerView(item), Token: token}, nil
}

func (s *Service) DeleteProjectRunner(ctx context.Context, projectID, runnerID int64) (RunnerView, error) {
	item, err := s.runnerRepo.GetByProjectAndID(ctx, projectID, runnerID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return RunnerView{}, apperror.NotFound("project runner not found", err)
		}
		return RunnerView{}, oops.In("runner").With("project_id", projectID, "runner_id", runnerID).Wrapf(err, "load project runner")
	}
	if deleteErr := s.runnerRepo.DeleteByID(ctx, item.ID); deleteErr != nil {
		return RunnerView{}, oops.In("runner").With("project_id", projectID, "runner_id", item.ID).Wrapf(deleteErr, "delete project runner")
	}
	return toRunnerView(item), nil
}

func (s *Service) Heartbeat(ctx context.Context, token string) (RunnerView, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return RunnerView{}, err
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return RunnerView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	runner, err = s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		return RunnerView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(err, "reload runner")
	}
	return toRunnerView(runner), nil
}

func (s *Service) ClaimJob(ctx context.Context, token string, lease time.Duration) (ClaimView, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return ClaimView{}, err
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return ClaimView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	job, claimed, err := s.jobService.ClaimProjectJobMatching(ctx, runner.ProjectID, runnerWorkerID(runner), lease, func(job cidomain.ProjectJob) (bool, error) {
		return runnerMatchesJob(runner, job)
	})
	if err != nil {
		return ClaimView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(err, "claim project job")
	}
	if claimed {
		if refreshErr := s.refreshPipelineForJob(ctx, runner.ProjectID, job.ID); refreshErr != nil {
			return ClaimView{}, refreshErr
		}
	}
	runner, err = s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		return ClaimView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(err, "reload runner")
	}
	return ClaimView{Claimed: claimed, Runner: toRunnerView(runner), Job: job}, nil
}

func (s *Service) CompleteJob(ctx context.Context, token string, jobID int64, result string) (cidomain.ProjectJob, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "load project job")
	}
	if ownershipErr := ensureRunnerCanUseJob(runner, job, time.Now().UTC()); ownershipErr != nil {
		return cidomain.ProjectJob{}, ownershipErr
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	completed, err := s.jobService.CompleteProjectJob(ctx, runner.ProjectID, jobID, result)
	if err != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "complete project job")
	}
	if refreshErr := s.refreshPipelineForJob(ctx, runner.ProjectID, jobID); refreshErr != nil {
		return cidomain.ProjectJob{}, refreshErr
	}
	return completed, nil
}

func (s *Service) FailJob(ctx context.Context, token string, jobID int64, message, result string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "load project job")
	}
	if ownershipErr := ensureRunnerCanUseJob(runner, job, time.Now().UTC()); ownershipErr != nil {
		return cidomain.ProjectJob{}, ownershipErr
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	failed, err := s.jobService.FailProjectJobWithResult(ctx, runner.ProjectID, jobID, message, result, retryAfter)
	if err != nil {
		return cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "fail project job")
	}
	if refreshErr := s.refreshPipelineForJob(ctx, runner.ProjectID, jobID); refreshErr != nil {
		return cidomain.ProjectJob{}, refreshErr
	}
	return failed, nil
}

func (s *Service) UploadArtifact(ctx context.Context, token string, jobID int64, input UploadArtifactInput) (cidomain.ProjectJobArtifact, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "load project job")
	}
	if ownershipErr := ensureRunnerCanUseJob(runner, job, time.Now().UTC()); ownershipErr != nil {
		return cidomain.ProjectJobArtifact{}, ownershipErr
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	artifact, err := s.jobService.UploadProjectJobArtifact(ctx, runner.ProjectID, jobID, jobservice.UploadArtifactInput{
		Name:          input.Name,
		FileName:      input.FileName,
		FilePath:      input.FilePath,
		ContentType:   input.ContentType,
		ContentBase64: input.ContentBase64,
	})
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "upload project job artifact")
	}
	return artifact, nil
}

func (s *Service) AppendTrace(ctx context.Context, token string, jobID int64, input AppendTraceInput) (jobservice.ProjectJobTrace, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return jobservice.ProjectJobTrace{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return jobservice.ProjectJobTrace{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "load project job")
	}
	if ownershipErr := ensureRunnerCanUseJob(runner, job, time.Now().UTC()); ownershipErr != nil {
		return jobservice.ProjectJobTrace{}, ownershipErr
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return jobservice.ProjectJobTrace{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	trace, err := s.jobService.AppendProjectJobTrace(ctx, runner.ProjectID, jobID, jobservice.AppendTraceInput{
		Output:          input.Output,
		OutputTruncated: input.OutputTruncated,
		DurationMillis:  input.DurationMillis,
	})
	if err != nil {
		return jobservice.ProjectJobTrace{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "append project job trace")
	}
	return trace, nil
}
