package mergerequest

import (
	"context"
	"errors"
	"fmt"
	"log/slog"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/samber/oops"
)

func (s *Service) triggerTargetBranchPipeline(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) {
	if s.pipelineSvc == nil {
		return
	}
	branch, err := s.resolveBranch(ctx, project, mr.TargetBranch)
	if err != nil || branch.Hash == "" {
		return
	}
	if _, _, err := s.pipelineSvc.CreatePushPipeline(ctx, project.ID, branch.Name, branch.Hash); err != nil {
		wrapped := oops.In("merge_request").With("project_id", project.ID, "merge_request_id", mr.ID, "target_branch", branch.Name, "commit_sha", branch.Hash).Wrapf(err, "trigger target branch pipeline")
		slog.Default().Warn("trigger target branch pipeline failed", slog.String("error", wrapped.Error()))
	}
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
		if errors.Is(err, gitports.ErrNotFound) {
			view.Status = "missing"
			view.BlockingReason = "required pipeline is missing"
			return view, nil
		}
		return CheckStatusView{}, oops.In("merge_request").With("project_id", project.ID, "merge_request_id", mr.ID, "ref", branch.Name, "commit_sha", branch.Hash).Wrapf(err, "load merge request pipeline")
	}
	view.Pipeline = &pipeline
	view.Status = pipeline.Status
	view.Mergeable = pipeline.Status == gitports.ProjectPipelineStatusSucceeded
	if !view.Mergeable {
		view.BlockingReason = "pipeline status is " + pipeline.Status
	}
	return view, nil
}

func (s *Service) hasCIConfig(ctx context.Context, project projectdomain.Project, commitSHA string) (bool, error) {
	_, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", commitSHA, project.DefaultBranch, defaultCIConfigPath)
	if err == nil {
		return true, nil
	}
	if errors.Is(err, gitports.ErrPathNotFound) || errors.Is(err, gitports.ErrReferenceNotFound) || errors.Is(err, gitports.ErrEmptyRepository) {
		return false, nil
	}
	return false, oops.In("merge_request").With("project_id", project.ID, "commit_sha", commitSHA, "path", defaultCIConfigPath).Wrapf(err, "check ci config")
}

func (s *Service) resolveBranch(ctx context.Context, project projectdomain.Project, branch string) (gitports.Branch, error) {
	branches, err := s.gitRepo.ListBranches(ctx, project.FullPath+".git", project.DefaultBranch)
	if err != nil {
		return gitports.Branch{}, oops.In("merge_request").With("project_id", project.ID, "branch", branch).Wrapf(err, "list branches")
	}
	for _, item := range branches {
		if item.Name == branch {
			return item, nil
		}
	}
	return gitports.Branch{}, apperror.NotFound("merge request branch not found", fmt.Errorf("branch %s not found", branch))
}

func (s *Service) ensureBranchExists(ctx context.Context, project projectdomain.Project, branch string) error {
	_, err := s.resolveBranch(ctx, project, branch)
	return err
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrMergeConflict):
		return apperror.Conflict("merge conflict", err)
	case errors.Is(err, gitports.ErrSourceReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrInvalidBranchName):
		return apperror.BadRequest("invalid branch name", err)
	default:
		return err
	}
}
