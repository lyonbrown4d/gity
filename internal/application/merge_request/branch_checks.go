package mergerequest

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/bmatcuk/doublestar/v4"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

func branchPatternMatches(pattern, branch string) bool {
	pattern = strings.TrimSpace(pattern)
	branch = strings.TrimSpace(branch)
	if pattern == "" || pattern == "*" {
		return true
	}
	matched, err := doublestar.Match(pattern, branch)
	if err == nil {
		return matched
	}
	return pattern == branch
}

func (s *Service) targetBranchProtection(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, bool, error) {
	if s.branchRepo == nil {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	protection, err := s.branchRepo.MatchByProjectAndBranch(ctx, projectID, branchName)
	if err == nil {
		return protection, true, nil
	}
	if errors.Is(err, gitports.ErrNotFound) {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	return projectdomain.ProjectBranchProtection{}, false, oops.In("merge_request").With("project_id", projectID, "target_branch", branchName).Wrapf(err, "check target branch protection")
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
