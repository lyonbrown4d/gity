package release

import (
	"errors"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
)

func mapGitError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrRepositoryNotFound):
		return apperror.NotFound("repository not found", err)
	case errors.Is(err, gitports.ErrReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrEmptyRepository):
		return apperror.NotFound("repository has no commits", err)
	default:
		return err
	}
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrTagExists):
		return apperror.Conflict("tag already exists", err)
	case errors.Is(err, gitports.ErrInvalidTagName):
		return apperror.BadRequest("invalid tag name", err)
	case errors.Is(err, gitports.ErrSourceReferenceNotFound), errors.Is(err, gitports.ErrReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	default:
		return err
	}
}
