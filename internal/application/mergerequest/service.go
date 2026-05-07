package mergerequest

import (
	"context"
	"errors"
	"fmt"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectmergerequest"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpipeline"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
	"net/http"
	"strings"
)

type Service struct {
	projectRepo  *projectrepo.Repository
	mergeRepo    *projectmergerequestrepo.Repository
	userRepo     *userrepo.Repository
	gitRepo      *gitrepo.Service
	gitRunner    *gitexec.Runner
	pipelineRepo *projectpipelinerepo.Repository
	pipelineSvc  *pipelineservice.Service
}

type PipelineDeps struct {
	pipelineRepo *projectpipelinerepo.Repository
	pipelineSvc  *pipelineservice.Service
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
	MergeRequest    mergedomain.ProjectMergeRequest `json:"merge_request"`
	SourceBranch    string                          `json:"source_branch"`
	SourceCommitSHA string                          `json:"source_commit_sha"`
	Required        bool                            `json:"required"`
	Mergeable       bool                            `json:"mergeable"`
	Status          string                          `json:"status"`
	BlockingReason  string                          `json:"blocking_reason,omitempty"`
	Pipeline        *cidomain.ProjectPipeline       `json:"pipeline,omitempty"`
}

type MergeInput struct {
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
}

const defaultCIConfigPath = ".gity-ci.plano"

func NewPipelineDeps(pipelineRepo *projectpipelinerepo.Repository, pipelineSvc *pipelineservice.Service) *PipelineDeps {
	return &PipelineDeps{pipelineRepo: pipelineRepo, pipelineSvc: pipelineSvc}
}

func NewService(projectRepo *projectrepo.Repository, mergeRepo *projectmergerequestrepo.Repository, userRepo *userrepo.Repository, gitRepo *gitrepo.Service, gitRunner *gitexec.Runner, pipelineDeps *PipelineDeps) *Service {
	service := &Service{projectRepo: projectRepo, mergeRepo: mergeRepo, userRepo: userRepo, gitRepo: gitRepo, gitRunner: gitRunner}
	if pipelineDeps != nil {
		service.pipelineRepo = pipelineDeps.pipelineRepo
		service.pipelineSvc = pipelineDeps.pipelineSvc
	}
	return service
}

func (s *Service) List(ctx context.Context, projectID int64) ([]mergedomain.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.mergeRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetByIID(ctx context.Context, projectID int64, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) Create(ctx context.Context, projectID int64, input CreateInput) (mergedomain.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "merge request author not found", err)
	}
	if strings.TrimSpace(input.Title) == "" {
		return mergedomain.ProjectMergeRequest{}, fmt.Errorf("merge request title is required")
	}
	source := strings.TrimSpace(input.SourceBranch)
	target := strings.TrimSpace(input.TargetBranch)
	if source == "" || target == "" {
		return mergedomain.ProjectMergeRequest{}, fmt.Errorf("source_branch and target_branch are required")
	}
	if source == target {
		return mergedomain.ProjectMergeRequest{}, fmt.Errorf("source_branch and target_branch must be different")
	}
	if err := s.ensureBranchExists(ctx, project, source); err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if err := s.ensureBranchExists(ctx, project, target); err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	return s.mergeRepo.Create(ctx, projectmergerequestrepo.CreateInput{
		ProjectID:    projectID,
		AuthorUserID: input.AuthorUserID,
		Title:        input.Title,
		Description:  input.Description,
		SourceBranch: source,
		TargetBranch: target,
	})
}

func (s *Service) Update(ctx context.Context, projectID int64, mergeIID int64, input UpdateInput) (mergedomain.ProjectMergeRequest, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return mergedomain.ProjectMergeRequest{}, fmt.Errorf("merge request title is required")
	}
	if input.State != nil {
		state := strings.TrimSpace(*input.State)
		if state != "opened" && state != "closed" && state != "merged" {
			return mergedomain.ProjectMergeRequest{}, fmt.Errorf("merge request state must be opened, closed, or merged")
		}
	}
	if err := s.mergeRepo.UpdateByID(ctx, mr.ID, projectmergerequestrepo.UpdateInput{Title: input.Title, Description: input.Description, State: input.State}); err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) GetDiff(ctx context.Context, projectID int64, mergeIID int64) (DiffView, error) {
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

func (s *Service) GetChecks(ctx context.Context, projectID int64, mergeIID int64) (CheckStatusView, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return CheckStatusView{}, err
	}
	return s.evaluateChecks(ctx, project, mr)
}

func (s *Service) Merge(ctx context.Context, projectID int64, mergeIID int64, input MergeInput) (mergedomain.ProjectMergeRequest, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if mr.State != "opened" {
		return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusConflict, "merge request is not opened", fmt.Errorf("merge request state: %s", mr.State))
	}
	checks, err := s.evaluateChecks(ctx, project, mr)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if checks.Required && !checks.Mergeable {
		return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusConflict, "merge request pipeline is not successful", errors.New(checks.BlockingReason))
	}
	message := strings.TrimSpace(input.Message)
	if message == "" {
		message = fmt.Sprintf("Merge branch '%s' into '%s'", mr.SourceBranch, mr.TargetBranch)
	}
	err = s.gitRunner.MergeBranches(ctx, project.FullPath+".git", gitexec.MergeBranchesInput{
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
	if err := s.mergeRepo.UpdateByID(ctx, mr.ID, projectmergerequestrepo.UpdateInput{State: &merged}); err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	s.triggerTargetBranchPipeline(ctx, project, mr)
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) triggerTargetBranchPipeline(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) {
	if s.pipelineSvc == nil {
		return
	}
	branch, err := s.resolveBranch(ctx, project, mr.TargetBranch)
	if err != nil || branch.Hash == "" {
		return
	}
	_, _, _ = s.pipelineSvc.CreatePushPipeline(ctx, project.ID, branch.Name, branch.Hash)
}

func (s *Service) evaluateChecks(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) (CheckStatusView, error) {
	branch, err := s.resolveBranch(ctx, project, mr.SourceBranch)
	if err != nil {
		return CheckStatusView{}, err
	}
	view := CheckStatusView{
		MergeRequest:    mr,
		SourceBranch:    branch.Name,
		SourceCommitSHA: branch.Hash,
		Mergeable:       true,
		Status:          "not_required",
	}
	required, err := s.hasCIConfig(ctx, project, branch.Hash)
	if err != nil {
		return CheckStatusView{}, err
	}
	if !required {
		return view, nil
	}
	view.Required = true
	view.Mergeable = false
	if s.pipelineRepo == nil {
		view.Status = "missing"
		view.BlockingReason = "pipeline repository is not configured"
		return view, nil
	}
	pipeline, err := s.pipelineRepo.GetLatestByProjectRefCommit(ctx, project.ID, branch.Name, branch.Hash)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			view.Status = "missing"
			view.BlockingReason = "required pipeline is missing"
			return view, nil
		}
		return CheckStatusView{}, err
	}
	view.Pipeline = &pipeline
	view.Status = pipeline.Status
	view.Mergeable = pipeline.Status == projectpipelinerepo.StatusSucceeded
	if !view.Mergeable {
		view.BlockingReason = fmt.Sprintf("pipeline status is %s", pipeline.Status)
	}
	return view, nil
}

func (s *Service) hasCIConfig(ctx context.Context, project projectdomain.Project, commitSHA string) (bool, error) {
	_, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", commitSHA, project.DefaultBranch, defaultCIConfigPath)
	if err == nil {
		return true, nil
	}
	if errors.Is(err, gitrepo.ErrPathNotFound) || errors.Is(err, gitrepo.ErrReferenceNotFound) || errors.Is(err, gitrepo.ErrEmptyRepository) {
		return false, nil
	}
	return false, err
}

func (s *Service) resolveBranch(ctx context.Context, project projectdomain.Project, branch string) (gitrepo.Branch, error) {
	branches, err := s.gitRepo.ListBranches(ctx, project.FullPath+".git", project.DefaultBranch)
	if err != nil {
		return gitrepo.Branch{}, err
	}
	for _, item := range branches {
		if item.Name == branch {
			return item, nil
		}
	}
	return gitrepo.Branch{}, httpx.NewError(http.StatusNotFound, "merge request branch not found", fmt.Errorf("branch %s not found", branch))
}

func (s *Service) ensureBranchExists(ctx context.Context, project projectdomain.Project, branch string) error {
	_, err := s.resolveBranch(ctx, project, branch)
	return err
}

func (s *Service) loadMergeRequest(ctx context.Context, projectID int64, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	_, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	return mr, err
}

func (s *Service) loadProjectMergeRequest(ctx context.Context, projectID int64, mergeIID int64) (projectdomain.Project, mergedomain.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	mr, err := s.loadMergeRequestRecord(ctx, projectID, mergeIID)
	if err != nil {
		return projectdomain.Project{}, mergedomain.ProjectMergeRequest{}, err
	}
	return project, mr, nil
}

func (s *Service) loadMergeRequestRecord(ctx context.Context, projectID int64, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	mr, err := s.mergeRepo.GetByProjectAndIID(ctx, projectID, mergeIID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return mergedomain.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "merge request not found", err)
		}
		return mergedomain.ProjectMergeRequest{}, err
	}
	return mr, nil
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitexec.ErrMergeConflict):
		return httpx.NewError(http.StatusConflict, "merge conflict", err)
	case errors.Is(err, gitexec.ErrSourceReferenceNotFound):
		return httpx.NewError(http.StatusNotFound, "git reference not found", err)
	case errors.Is(err, gitexec.ErrInvalidBranchName):
		return httpx.NewError(http.StatusBadRequest, "invalid branch name", err)
	default:
		return err
	}
}
