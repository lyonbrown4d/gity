package lfs

import (
	"log/slog"
	"net/http"
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v3"
	lfsservice "github.com/lyonbrown4d/gity/internal/application/lfs"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
)

func RegisterRoutes(app *fiber.App, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) {
	app.Use(func(c fiber.Ctx) error {
		if !isLFSPath(c.Path(), c.Method()) {
			return nextRoute(c)
		}
		switch c.Method() {
		case fiber.MethodPost:
			return handlePost(c, logger, authRuntime, repo, service)
		case fiber.MethodPut:
			return handleUpload(c, logger, authRuntime, repo, service)
		case fiber.MethodGet:
			return handleGet(c, logger, authRuntime, repo, service)
		default:
			return nextRoute(c)
		}
	})
}

func handlePost(c fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	switch {
	case strings.HasSuffix(path, "/info/lfs/objects/batch"):
		return handleBatch(c, logger, authRuntime, repo, service)
	case strings.HasSuffix(path, "/info/lfs/locks/verify"):
		return handleVerifyLocks(c, logger, authRuntime, repo, service)
	case strings.HasSuffix(path, "/info/lfs/locks"):
		return handleCreateLock(c, logger, authRuntime, repo, service)
	case strings.Contains(path, "/info/lfs/locks/") && strings.HasSuffix(path, "/unlock"):
		return handleUnlockLock(c, logger, authRuntime, repo, service)
	default:
		return nextRoute(c)
	}
}

func handleGet(c fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	if strings.HasSuffix(path, "/info/lfs/locks") {
		return handleListLocks(c, logger, authRuntime, repo, service)
	}
	return handleDownload(c, logger, authRuntime, repo, service)
}

func handleBatch(c fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/objects/batch")
	project, err := loadProject(c.Context(), repo, repoPath)
	if err != nil {
		return err
	}

	var request lfsservice.BatchRequest
	if parseErr := c.Bind().Body(&request); parseErr != nil {
		return fiber.NewError(http.StatusBadRequest, "invalid lfs batch request")
	}
	if authErr := authorizeLFSOperation(c, authRuntime, project, request.Operation); authErr != nil {
		return authErr
	}

	response, err := service.PrepareBatch(c.Context(), project.ID, request, strings.TrimRight(c.BaseURL(), "/"), strings.Trim(repoPath, "/"))
	if err != nil {
		logger.Error("lfs batch failed", slog.String("project", project.FullPath), slog.String("operation", request.Operation), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return sendJSON(c, http.StatusOK, response)
}

func handleUpload(c fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	project, oid, err := loadLFSObjectTarget(c, repo)
	if err != nil {
		return err
	}
	if authErr := authorizeLFSOperation(c, authRuntime, project, "upload"); authErr != nil {
		return authErr
	}

	_, err = service.UploadObject(c.Context(), project.ID, oid, c.Body())
	if err != nil {
		logger.Error("lfs upload failed", slog.String("project", project.FullPath), slog.String("oid", oid), slog.String("error", err.Error()))
		return err
	}
	return sendStatus(c, http.StatusOK)
}

func handleDownload(c fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	project, oid, err := loadLFSObjectTarget(c, repo)
	if err != nil {
		return err
	}
	if authErr := authorizeLFSOperation(c, authRuntime, project, "download"); authErr != nil {
		return authErr
	}

	result, err := service.DownloadObject(c.Context(), project.ID, oid)
	if err != nil {
		logger.Error("lfs download failed", slog.String("project", project.FullPath), slog.String("oid", oid), slog.String("error", err.Error()))
		return err
	}

	c.Set(fiber.HeaderContentType, "application/octet-stream")
	c.Set(fiber.HeaderContentLength, strconv.Itoa(len(result.Content)))
	return sendBytes(c, result.Content)
}
