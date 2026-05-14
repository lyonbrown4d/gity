package pipeline

import (
	"context"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

type Service struct {
	projectRepo     gitports.ProjectRepository
	pipelineRepo    gitports.ProjectPipelineRepository
	pipelineJobRepo gitports.ProjectPipelineJobRepository
	jobService      *jobservice.Service
	jobRepo         gitports.ProjectJobRepository
	gitRepo         gitports.GitRepository
	variableRepo    gitports.ProjectCIVariableRepository
}

type RuntimeDeps struct {
	jobRepo      gitports.ProjectJobRepository
	gitRepo      gitports.GitRepository
	variableRepo gitports.ProjectCIVariableRepository
}

func NewRuntimeDeps(jobRepo gitports.ProjectJobRepository, gitRepo gitports.GitRepository, variableRepo gitports.ProjectCIVariableRepository) RuntimeDeps {
	return RuntimeDeps{jobRepo: jobRepo, gitRepo: gitRepo, variableRepo: variableRepo}
}

func NewService(
	projectRepo gitports.ProjectRepository,
	pipelineRepo gitports.ProjectPipelineRepository,
	pipelineJobRepo gitports.ProjectPipelineJobRepository,
	jobService *jobservice.Service,
	jobRepo gitports.ProjectJobRepository,
	gitRepo gitports.GitRepository,
	variableRepo gitports.ProjectCIVariableRepository,
) *Service {
	return NewServiceFromDeps(projectRepo, pipelineRepo, pipelineJobRepo, jobService, RuntimeDeps{jobRepo: jobRepo, gitRepo: gitRepo, variableRepo: variableRepo})
}

func NewServiceFromDeps(
	projectRepo gitports.ProjectRepository,
	pipelineRepo gitports.ProjectPipelineRepository,
	pipelineJobRepo gitports.ProjectPipelineJobRepository,
	jobService *jobservice.Service,
	deps RuntimeDeps,
) *Service {
	return &Service{
		projectRepo:     projectRepo,
		pipelineRepo:    pipelineRepo,
		pipelineJobRepo: pipelineJobRepo,
		jobService:      jobService,
		jobRepo:         deps.jobRepo,
		gitRepo:         deps.gitRepo,
		variableRepo:    deps.variableRepo,
	}
}

func (s *Service) ListPipelines(ctx context.Context, projectID int64) ([]cidomain.ProjectPipeline, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.pipelineRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("pipeline").With("project_id", projectID).Wrapf(err, "list project pipelines")
	}
	return items.Values(), nil
}

func (s *Service) GetPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	item, err := s.refreshPipelineState(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	jobs, err := s.listPipelineJobs(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	return PipelineView{Pipeline: item, Jobs: jobs}, nil
}

func (s *Service) ListPipelineJobs(ctx context.Context, projectID, pipelineID int64) ([]PipelineJobView, error) {
	if _, err := s.refreshPipelineState(ctx, projectID, pipelineID); err != nil {
		return nil, err
	}
	return s.listPipelineJobs(ctx, projectID, pipelineID)
}

func (s *Service) RefreshPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	pipeline, err := s.refreshPipelineState(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	jobs, err := s.listPipelineJobs(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	return PipelineView{Pipeline: pipeline, Jobs: jobs}, nil
}
