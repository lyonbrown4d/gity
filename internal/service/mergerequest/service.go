package mergerequest

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"

	"github.com/DaiYuANg/gity/internal/entity"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
)

type Service struct {
	projectRepo *projectrepo.Repository
	mergeRepo   *projectmergerequestrepo.Repository
	userRepo    *userrepo.Repository
	gitRepo     *gitrepo.Service
	gitRunner   *gitexec.Runner
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
	MergeRequest entity.ProjectMergeRequest `json:"merge_request"`
	Diff         string                     `json:"diff"`
}

type MergeInput struct {
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
}

func NewService(projectRepo *projectrepo.Repository, mergeRepo *projectmergerequestrepo.Repository, userRepo *userrepo.Repository, gitRepo *gitrepo.Service, gitRunner *gitexec.Runner) *Service {
	return &Service{projectRepo: projectRepo, mergeRepo: mergeRepo, userRepo: userRepo, gitRepo: gitRepo, gitRunner: gitRunner}
}

func (s *Service) List(ctx context.Context, projectID int64) ([]entity.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.mergeRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetByIID(ctx context.Context, projectID int64, mergeIID int64) (entity.ProjectMergeRequest, error) {
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) Create(ctx context.Context, projectID int64, input CreateInput) (entity.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return entity.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return entity.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "merge request author not found", err)
	}
	if strings.TrimSpace(input.Title) == "" {
		return entity.ProjectMergeRequest{}, fmt.Errorf("merge request title is required")
	}
	source := strings.TrimSpace(input.SourceBranch)
	target := strings.TrimSpace(input.TargetBranch)
	if source == "" || target == "" {
		return entity.ProjectMergeRequest{}, fmt.Errorf("source_branch and target_branch are required")
	}
	if source == target {
		return entity.ProjectMergeRequest{}, fmt.Errorf("source_branch and target_branch must be different")
	}
	if err := s.ensureBranchExists(ctx, project, source); err != nil {
		return entity.ProjectMergeRequest{}, err
	}
	if err := s.ensureBranchExists(ctx, project, target); err != nil {
		return entity.ProjectMergeRequest{}, err
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

func (s *Service) Update(ctx context.Context, projectID int64, mergeIID int64, input UpdateInput) (entity.ProjectMergeRequest, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return entity.ProjectMergeRequest{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return entity.ProjectMergeRequest{}, fmt.Errorf("merge request title is required")
	}
	if input.State != nil {
		state := strings.TrimSpace(*input.State)
		if state != "opened" && state != "closed" && state != "merged" {
			return entity.ProjectMergeRequest{}, fmt.Errorf("merge request state must be opened, closed, or merged")
		}
	}
	if err := s.mergeRepo.UpdateByID(ctx, mr.ID, projectmergerequestrepo.UpdateInput{Title: input.Title, Description: input.Description, State: input.State}); err != nil {
		return entity.ProjectMergeRequest{}, err
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

func (s *Service) Merge(ctx context.Context, projectID int64, mergeIID int64, input MergeInput) (entity.ProjectMergeRequest, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return entity.ProjectMergeRequest{}, err
	}
	if mr.State != "opened" {
		return entity.ProjectMergeRequest{}, httpx.NewError(http.StatusConflict, "merge request is not opened", fmt.Errorf("merge request state: %s", mr.State))
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
		return entity.ProjectMergeRequest{}, mapGitExecError(err)
	}
	merged := "merged"
	if err := s.mergeRepo.UpdateByID(ctx, mr.ID, projectmergerequestrepo.UpdateInput{State: &merged}); err != nil {
		return entity.ProjectMergeRequest{}, err
	}
	return s.loadMergeRequest(ctx, projectID, mergeIID)
}

func (s *Service) ensureBranchExists(ctx context.Context, project entity.Project, branch string) error {
	branches, err := s.gitRepo.ListBranches(ctx, project.FullPath+".git", project.DefaultBranch)
	if err != nil {
		return err
	}
	for _, item := range branches {
		if item.Name == branch {
			return nil
		}
	}
	return httpx.NewError(http.StatusNotFound, "merge request branch not found", fmt.Errorf("branch %s not found", branch))
}

func (s *Service) loadMergeRequest(ctx context.Context, projectID int64, mergeIID int64) (entity.ProjectMergeRequest, error) {
	_, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	return mr, err
}

func (s *Service) loadProjectMergeRequest(ctx context.Context, projectID int64, mergeIID int64) (entity.Project, entity.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return entity.Project{}, entity.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	mr, err := s.loadMergeRequestRecord(ctx, projectID, mergeIID)
	if err != nil {
		return entity.Project{}, entity.ProjectMergeRequest{}, err
	}
	return project, mr, nil
}

func (s *Service) loadMergeRequestRecord(ctx context.Context, projectID int64, mergeIID int64) (entity.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	mr, err := s.mergeRepo.GetByProjectAndIID(ctx, projectID, mergeIID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return entity.ProjectMergeRequest{}, httpx.NewError(http.StatusNotFound, "merge request not found", err)
		}
		return entity.ProjectMergeRequest{}, err
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
