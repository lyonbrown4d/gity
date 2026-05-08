package gittransport

import (
	"context"
	"errors"
	"net/http"

	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/gofiber/fiber/v2"
)

func rejectProtectedBranchUpdates(ctx context.Context, project projectView, repo *projectbranchprotectionrepo.Repository, updates []receivePackUpdate) error {
	for _, update := range updates {
		if update.BranchName == "" {
			continue
		}
		if _, err := repo.GetByProjectAndBranch(ctx, project.ID, update.BranchName); err == nil {
			return fiber.NewError(http.StatusForbidden, "protected branch cannot be updated: "+update.BranchName)
		} else if !errors.Is(err, dbxrepo.ErrNotFound) {
			return fiber.NewError(http.StatusInternalServerError, "check branch protection failed")
		}
	}
	return nil
}
