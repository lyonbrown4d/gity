package lfs

import (
	"log/slog"
	"net/http"
	"strings"

	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	"github.com/gofiber/fiber/v2"
)

func handleCreateLock(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/locks")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	principal, err := requireProjectWritePrincipal(c, authRuntime, project)
	if err != nil {
		return err
	}

	var request lfsservice.CreateLockInput
	if parseErr := c.BodyParser(&request); parseErr != nil {
		return fiber.NewError(http.StatusBadRequest, "invalid lfs lock request")
	}
	response, err := service.CreateLock(c.UserContext(), project.ID, principal.UserID, request)
	if err != nil {
		logger.Error("lfs create lock failed", slog.String("project", project.FullPath), slog.String("path", request.Path), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return c.Status(http.StatusCreated).JSON(response)
}

func handleListLocks(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/locks")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	if _, authErr := requireProjectWritePrincipal(c, authRuntime, project); authErr != nil {
		return authErr
	}
	limit, err := parseLimit(c.Query("limit"))
	if err != nil {
		return fiber.NewError(http.StatusBadRequest, "invalid lfs lock limit")
	}
	response, err := service.ListLocks(c.UserContext(), project.ID, lfsservice.LockListInput{Path: c.Query("path"), ID: c.Query("id"), Cursor: c.Query("cursor"), Limit: limit})
	if err != nil {
		logger.Error("lfs list locks failed", slog.String("project", project.FullPath), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return c.Status(http.StatusOK).JSON(response)
}

func handleVerifyLocks(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/locks/verify")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	principal, err := requireProjectWritePrincipal(c, authRuntime, project)
	if err != nil {
		return err
	}
	var request struct {
		Cursor string `json:"cursor"`
		Limit  int    `json:"limit"`
	}
	if len(c.Body()) > 0 {
		if parseErr := c.BodyParser(&request); parseErr != nil {
			return fiber.NewError(http.StatusBadRequest, "invalid lfs lock verify request")
		}
	}
	response, err := service.VerifyLocks(c.UserContext(), project.ID, principal.UserID, lfsservice.LockListInput{Cursor: request.Cursor, Limit: request.Limit})
	if err != nil {
		logger.Error("lfs verify locks failed", slog.String("project", project.FullPath), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return c.Status(http.StatusOK).JSON(response)
}

func handleUnlockLock(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	project, lockID, err := loadLFSUnlockTarget(c, repo)
	if err != nil {
		return err
	}
	principal, err := requireProjectWritePrincipal(c, authRuntime, project)
	if err != nil {
		return err
	}
	var request lfsservice.UnlockInput
	if len(c.Body()) > 0 {
		if parseErr := c.BodyParser(&request); parseErr != nil {
			return fiber.NewError(http.StatusBadRequest, "invalid lfs unlock request")
		}
	}
	response, err := service.Unlock(c.UserContext(), project.ID, principal.UserID, lockID, request)
	if err != nil {
		logger.Error("lfs unlock failed", slog.String("project", project.FullPath), slog.String("lock_id", lockID), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return c.Status(http.StatusOK).JSON(response)
}
