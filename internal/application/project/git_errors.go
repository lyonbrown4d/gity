package project

import (
	"errors"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
)

func repositoryPath(project projectdomain.Project) string {
	return project.FullPath + ".git"
}

func mapGitError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrRepositoryNotFound):
		return apperror.NotFound("repository not found", err)
	case errors.Is(err, gitports.ErrReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrEmptyRepository):
		return apperror.NotFound("repository has no commits", err)
	case errors.Is(err, gitports.ErrInvalidSearchQuery):
		return apperror.BadRequest("invalid search query", err)
	case errors.Is(err, gitports.ErrInvalidSearchRegexp):
		return apperror.BadRequest("invalid search regex", err)
	case errors.Is(err, gitports.ErrPathNotFound):
		return apperror.NotFound("repository path not found", err)
	case errors.Is(err, gitports.ErrReadmeNotFound):
		return apperror.NotFound("repository readme not found", err)
	default:
		return err
	}
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrBranchExists):
		return apperror.Conflict("branch already exists", err)
	case errors.Is(err, gitports.ErrInvalidBranchName):
		return apperror.BadRequest("invalid branch name", err)
	case errors.Is(err, gitports.ErrSourceReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrFileAlreadyExists):
		return apperror.Conflict("repository file already exists", err)
	default:
		return err
	}
}
