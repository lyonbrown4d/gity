package mergerequest

import (
	"context"
	"errors"
	"fmt"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	"github.com/samber/oops"
)

type Service struct {
	projectRepo     gitports.ProjectRepository
	mergeRepo       gitports.ProjectMergeRequestRepository
	participantRepo gitports.ProjectMergeRequestParticipantRepository
	userRepo        gitports.UserRepository
	branchRepo      gitports.ProjectBranchProtectionRepository
	gitRepo         gitports.GitRepository
	gitRunner       gitports.GitRunner
	pipelineRepo    gitports.ProjectPipelineRepository
	pipelineSvc     *pipelineservice.Service
	events          gitports.DomainEventPublisher
}

type PipelineDeps struct {
	pipelineRepo gitports.ProjectPipelineRepository
	pipelineSvc  *pipelineservice.Service
}

type GitDependencies struct {
	gitRepo   gitports.GitRepository
	gitRunner gitports.GitRunner
}

type Repositories struct {
	projectRepo     gitports.ProjectRepository
	mergeRepo       gitports.ProjectMergeRequestRepository
	participantRepo gitports.ProjectMergeRequestParticipantRepository
	userRepo        gitports.UserRepository
	branchRepo      gitports.ProjectBranchProtectionRepository
}

type CreateInput struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
	SourceBranch string `json:"source_branch"`
	TargetBranch string `json:"target_branch"`
}

type UpdateInput struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
}

type DiffView struct {
	MergeRequest mergedomain.ProjectMergeRequest `json:"merge_request"`
	Diff         string                          `json:"diff"`
}

type CheckStatusView struct {
	MergeRequest           mergedomain.ProjectMergeRequest `json:"merge_request"`
	SourceBranch           string                          `json:"source_branch"`
	SourceCommitSHA        string                          `json:"source_commit_sha"`
	TargetBranch           string                          `json:"target_branch"`
	TargetBranchProtected  bool                            `json:"target_branch_protected"`
	RequireMergeRequest    bool                            `json:"require_merge_request"`
	RequirePipelineSuccess bool                            `json:"require_pipeline_success"`
	PipelineRequired       bool                            `json:"pipeline_required"`
	Required               bool                            `json:"required"`
	Mergeable              bool                            `json:"mergeable"`
	Status                 string                          `json:"status"`
	BlockingReason         string                          `json:"blocking_reason,omitempty"`
	Pipeline               *cidomain.ProjectPipeline       `json:"pipeline,omitempty"`
}

type MergeInput struct {
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
	ActorUserID int64  `json:"actor_user_id"`
}

const defaultCIConfigPath = ".gity-ci.plano"

func NewPipelineDeps(pipelineRepo gitports.ProjectPipelineRepository, pipelineSvc *pipelineservice.Service) *PipelineDeps {
	return &PipelineDeps{pipelineRepo: pipelineRepo, pipelineSvc: pipelineSvc}
}

func NewGitDependencies(gitRepo gitports.GitRepository, gitRunner gitports.GitRunner) GitDependencies {
	return GitDependencies{gitRepo: gitRepo, gitRunner: gitRunner}
}

func NewRepositories(projectRepo gitports.ProjectRepository, mergeRepo gitports.ProjectMergeRequestRepository, participantRepo gitports.ProjectMergeRequestParticipantRepository, userRepo gitports.UserRepository, branchRepo gitports.ProjectBranchProtectionRepository) Repositories {
	return Repositories{projectRepo: projectRepo, mergeRepo: mergeRepo, participantRepo: participantRepo, userRepo: userRepo, branchRepo: branchRepo}
}

func NewService(projectRepo gitports.ProjectRepository, mergeRepo gitports.ProjectMergeRequestRepository, userRepo gitports.UserRepository, gitRepo gitports.GitRepository, gitRunner gitports.GitRunner, pipelineDeps *PipelineDeps) *Service {
	return NewServiceWithDependencies(
		NewRepositories(projectRepo, mergeRepo, nil, userRepo, nil),
		NewGitDependencies(gitRepo, gitRunner),
		pipelineDeps,
		gitports.NoopDomainEventPublisher{},
	)
}

func NewServiceWithDependencies(repos Repositories, git GitDependencies, pipelineDeps *PipelineDeps, events gitports.DomainEventPublisher) *Service {
	if events == nil {
		events = gitports.NoopDomainEventPublisher{}
	}
	service := &Service{
		projectRepo:     repos.projectRepo,
		mergeRepo:       repos.mergeRepo,
		participantRepo: repos.participantRepo,
		userRepo:        repos.userRepo,
		branchRepo:      repos.branchRepo,
		gitRepo:         git.gitRepo,
		gitRunner:       git.gitRunner,
	}
	service.events = events
	if pipelineDeps != nil {
		service.pipelineRepo = pipelineDeps.pipelineRepo
		service.pipelineSvc = pipelineDeps.pipelineSvc
	}
	return service
}

func (s *Service) List(ctx context.Context, projectID int64) ([]mergedomain.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.mergeRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("merge_request").With("project_id", projectID).Wrapf(err, "list merge requests")
	}
	return items.Values(), nil
}

func (s *Service) GetByIID(ctx context.Context, projectID, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) Create(ctx context.Context, projectID int64, input CreateInput) (mergedomain.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, apperror.NotFound("project not found", err)
	}
	if _, authorErr := s.userRepo.GetByID(ctx, input.AuthorUserID); authorErr != nil {
		return mergedomain.ProjectMergeRequest{}, apperror.NotFound("merge request author not found", authorErr)
	}
	if strings.TrimSpace(input.Title) == "" {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "author_user_id", input.AuthorUserID).New("merge request title is required")
	}
	source := strings.TrimSpace(input.SourceBranch)
	target := strings.TrimSpace(input.TargetBranch)
	if source == "" || target == "" {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "source_branch", source, "target_branch", target).New("source_branch and target_branch are required")
	}
	if source == target {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "source_branch", source, "target_branch", target).New("source_branch and target_branch must be different")
	}
	if sourceErr := s.ensureBranchExists(ctx, project, source); sourceErr != nil {
		return mergedomain.ProjectMergeRequest{}, sourceErr
	}
	if targetErr := s.ensureBranchExists(ctx, project, target); targetErr != nil {
		return mergedomain.ProjectMergeRequest{}, targetErr
	}
	mr, err := s.mergeRepo.Create(ctx, gitports.CreateProjectMergeRequestInput{
		ProjectID:    projectID,
		AuthorUserID: input.AuthorUserID,
		Title:        input.Title,
		Description:  input.Description,
		SourceBranch: source,
		TargetBranch: target,
	})
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "author_user_id", input.AuthorUserID, "source_branch", source, "target_branch", target).Wrapf(err, "create merge request")
	}
	return mr, nil
}

func (s *Service) Update(ctx context.Context, projectID, mergeIID int64, input UpdateInput) (mergedomain.ProjectMergeRequest, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return mergedomain.ProjectMergeRequest{}, errors.New("merge request title is required")
	}
	if input.State != nil {
		state := strings.TrimSpace(*input.State)
		if state != "opened" && state != "closed" && state != "merged" {
			return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "state", state).New("merge request state must be opened, closed, or merged")
		}
	}
	if err := s.mergeRepo.UpdateByID(ctx, mr.ID, gitports.UpdateProjectMergeRequestInput{Title: input.Title, Description: input.Description, State: input.State}); err != nil {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID).Wrapf(err, "update merge request")
	}
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) GetDiff(ctx context.Context, projectID, mergeIID int64) (DiffView, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return DiffView{}, err
	}
	diff, err := s.gitRunner.DiffBranches(ctx, project.FullPath+".git", mr.TargetBranch, mr.SourceBranch)
	if err != nil {
		return DiffView{}, mapGitExecError(err)
	}
	return DiffView{MergeRequest: mr, Diff: diff}, nil
}

func (s *Service) GetChecks(ctx context.Context, projectID, mergeIID int64) (CheckStatusView, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return CheckStatusView{}, err
	}
	return s.evaluateChecks(ctx, project, mr)
}

func (s *Service) Merge(ctx context.Context, projectID, mergeIID int64, input MergeInput) (mergedomain.ProjectMergeRequest, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if mr.State != "opened" {
		return mergedomain.ProjectMergeRequest{}, apperror.Conflict("merge request is not opened", fmt.Errorf("merge request state: %s", mr.State))
	}
	checks, err := s.evaluateChecks(ctx, project, mr)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if checks.Required && !checks.Mergeable {
		return mergedomain.ProjectMergeRequest{}, apperror.Conflict("merge request pipeline is not successful", errors.New(checks.BlockingReason))
	}
	message := strings.TrimSpace(input.Message)
	if message == "" {
		message = fmt.Sprintf("Merge branch '%s' into '%s'", mr.SourceBranch, mr.TargetBranch)
	}
	err = s.gitRunner.MergeBranches(ctx, project.FullPath+".git", gitports.MergeBranchesInput{
		TargetBranch: mr.TargetBranch,
		SourceBranch: mr.SourceBranch,
		Message:      message,
		AuthorName:   input.AuthorName,
		AuthorEmail:  input.AuthorEmail,
	})
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, mapGitExecError(err)
	}
	merged := "merged"
	if updateErr := s.mergeRepo.UpdateByID(ctx, mr.ID, gitports.UpdateProjectMergeRequestInput{State: &merged}); updateErr != nil {
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID).Wrapf(updateErr, "mark merge request merged")
	}
	s.triggerTargetBranchPipeline(ctx, project, mr)
	mergedMR, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	s.publishRepositoryChanged(ctx, project, mr.TargetBranch)
	s.publishEventAsync(ctx, mergedomain.NewProjectMergeRequestMergedEvent(mergedMR, input.ActorUserID))
	return mergedMR, nil
}
