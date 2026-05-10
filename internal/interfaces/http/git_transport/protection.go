package gittransport

import (
	"context"
	"errors"
	"net/http"

	projectports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	"github.com/gofiber/fiber/v2"
)

func rejectProtectedBranchUpdates(ctx context.Context, project projectView, repo *projectbranchprotectionrepo.Repository, updates []receivePackUpdate) error {
	for _, update := range updates {
		if err := rejectProtectedBranchUpdate(ctx, project, repo, update); err != nil {
			return err
		}
	}
	return nil
}

func protectedForcePushRefs(ctx context.Context, project projectView, repo *projectbranchprotectionrepo.Repository, updates []receivePackUpdate) ([]string, error) {
	refs := make([]string, 0, len(updates))
	for _, update := range updates {
		if update.BranchName == "" || update.Delete || isZeroOID(update.OldSHA) || isZeroOID(update.NewSHA) {
			continue
		}
		protection, protected, err := gitTransportBranchProtection(ctx, repo, project.ID, update.BranchName)
		if err != nil {
			return nil, err
		}
		if protected && protection.BlocksForcePush() {
			refs = append(refs, update.RefName)
		}
	}
	return refs, nil
}

func rejectProtectedBranchUpdate(ctx context.Context, project projectView, repo *projectbranchprotectionrepo.Repository, update receivePackUpdate) error {
	if update.BranchName == "" {
		return nil
	}
	if update.Delete && update.BranchName == project.DefaultBranch {
		return fiber.NewError(http.StatusForbidden, "default branch cannot be deleted: "+update.BranchName)
	}
	protection, protected, err := gitTransportBranchProtection(ctx, repo, project.ID, update.BranchName)
	if err != nil || !protected {
		return err
	}
	if update.Delete {
		return rejectProtectedBranchDelete(update, protection)
	}
	if protection.BlocksDirectPush() {
		return fiber.NewError(http.StatusForbidden, "protected branch cannot be updated: "+update.BranchName)
	}
	return nil
}

func gitTransportBranchProtection(ctx context.Context, repo *projectbranchprotectionrepo.Repository, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, bool, error) {
	protection, err := repo.MatchByProjectAndBranch(ctx, projectID, branchName)
	if err == nil {
		return protection, true, nil
	}
	if errors.Is(err, projectports.ErrNotFound) {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	return projectdomain.ProjectBranchProtection{}, false, fiber.NewError(http.StatusInternalServerError, "check branch protection failed")
}

func rejectProtectedBranchDelete(update receivePackUpdate, protection projectdomain.ProjectBranchProtection) error {
	if protection.BlocksDelete() {
		return fiber.NewError(http.StatusForbidden, "protected branch cannot be deleted: "+update.BranchName)
	}
	return nil
}
