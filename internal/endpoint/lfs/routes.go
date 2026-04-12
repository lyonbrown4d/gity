package lfs

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"strings"

	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	lfsservice "github.com/DaiYuANg/gity/internal/service/lfs"
	"github.com/gofiber/fiber/v2"
)

func RegisterRoutes(app *fiber.App, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) {
	app.Use(func(c *fiber.Ctx) error {
		if !isLFSPath(c.Path(), c.Method()) {
			return c.Next()
		}
		switch c.Method() {
		case fiber.MethodPost:
			return handlePost(c, logger, authRuntime, repo, service)
		case fiber.MethodPut:
			return handleUpload(c, logger, authRuntime, repo, service)
		case fiber.MethodGet:
			return handleGet(c, logger, authRuntime, repo, service)
		default:
			return c.Next()
		}
	})
}

func handlePost(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
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
		return c.Next()
	}
}

func handleGet(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	if strings.HasSuffix(path, "/info/lfs/locks") {
		return handleListLocks(c, logger, authRuntime, repo, service)
	}
	return handleDownload(c, logger, authRuntime, repo, service)
}

func handleBatch(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/objects/batch")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}

	var request lfsservice.BatchRequest
	if err := c.BodyParser(&request); err != nil {
		return fiber.NewError(http.StatusBadRequest, "invalid lfs batch request")
	}
	if err := authorizeLFSOperation(c, authRuntime, project, request.Operation); err != nil {
		return err
	}

	response, err := service.PrepareBatch(c.UserContext(), project.ID, request, strings.TrimRight(c.BaseURL(), "/"), strings.Trim(repoPath, "/"))
	if err != nil {
		logger.Error("lfs batch failed", slog.String("project", project.FullPath), slog.String("operation", request.Operation), slog.String("error", err.Error()))
		return err
	}
	c.Set(fiber.HeaderContentType, "application/vnd.git-lfs+json")
	return c.Status(http.StatusOK).JSON(response)
}

func handleUpload(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	project, oid, err := loadLFSObjectTarget(c, repo)
	if err != nil {
		return err
	}
	if err := authorizeLFSOperation(c, authRuntime, project, "upload"); err != nil {
		return err
	}

	_, err = service.UploadObject(c.UserContext(), project.ID, oid, c.Body())
	if err != nil {
		logger.Error("lfs upload failed", slog.String("project", project.FullPath), slog.String("oid", oid), slog.String("error", err.Error()))
		return err
	}
	return c.SendStatus(http.StatusOK)
}

func handleDownload(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	project, oid, err := loadLFSObjectTarget(c, repo)
	if err != nil {
		return err
	}
	if err := authorizeLFSOperation(c, authRuntime, project, "download"); err != nil {
		return err
	}

	result, err := service.DownloadObject(c.UserContext(), project.ID, oid)
	if err != nil {
		logger.Error("lfs download failed", slog.String("project", project.FullPath), slog.String("oid", oid), slog.String("error", err.Error()))
		return err
	}

	c.Set(fiber.HeaderContentType, "application/octet-stream")
	c.Set(fiber.HeaderContentLength, fmt.Sprintf("%d", len(result.Content)))
	return c.Send(result.Content)
}

func handleCreateLock(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
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
	if err := c.BodyParser(&request); err != nil {
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

func handleListLocks(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
	path := normalizeRequestPath(c.Path())
	repoPath := strings.TrimSuffix(path, "/info/lfs/locks")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	if _, err := requireProjectWritePrincipal(c, authRuntime, project); err != nil {
		return err
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

func handleVerifyLocks(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
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
		if err := c.BodyParser(&request); err != nil {
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

func handleUnlockLock(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, service *lfsservice.Service) error {
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
		if err := c.BodyParser(&request); err != nil {
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

func loadLFSObjectTarget(c *fiber.Ctx, repo *projectrepo.Repository) (projectView, string, error) {
	path := normalizeRequestPath(c.Path())
	idx := strings.Index(path, "/info/lfs/objects/")
	if idx <= 0 {
		return projectView{}, "", c.Next()
	}
	repoPath := path[:idx]
	oid := strings.TrimSpace(path[idx+len("/info/lfs/objects/"):])
	if oid == "" {
		return projectView{}, "", fiber.ErrNotFound
	}
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return projectView{}, "", err
	}
	return project, oid, nil
}

func loadLFSUnlockTarget(c *fiber.Ctx, repo *projectrepo.Repository) (projectView, string, error) {
	path := normalizeRequestPath(c.Path())
	idx := strings.Index(path, "/info/lfs/locks/")
	if idx <= 0 || !strings.HasSuffix(path, "/unlock") {
		return projectView{}, "", c.Next()
	}
	repoPath := path[:idx]
	lockID := strings.TrimSuffix(path[idx+len("/info/lfs/locks/"):], "/unlock")
	lockID = strings.TrimSpace(lockID)
	if lockID == "" {
		return projectView{}, "", fiber.ErrNotFound
	}
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return projectView{}, "", err
	}
	return project, lockID, nil
}

func authorizeLFSOperation(c *fiber.Ctx, authRuntime *platformauth.Runtime, project projectView, operation string) error {
	readOperation := strings.EqualFold(strings.TrimSpace(operation), "download")
	if readOperation && strings.TrimSpace(project.Visibility) == "public" {
		return nil
	}

	principal, ok, err := authRuntime.AuthenticateHeader(c.UserContext(), c.Get(fiber.HeaderAuthorization))
	if err != nil {
		return fiber.NewError(http.StatusUnauthorized, "invalid credentials")
	}
	if !ok {
		return fiber.NewError(http.StatusUnauthorized, "authentication required")
	}

	scope := platformauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
	allowed := false
	if readOperation {
		allowed, err = authRuntime.CanReadProject(c.UserContext(), principal, scope)
	} else {
		allowed, err = authRuntime.CanWriteProject(c.UserContext(), principal, scope)
	}
	if err != nil {
		return fiber.NewError(http.StatusForbidden, "authorization failed")
	}
	if !allowed {
		return fiber.NewError(http.StatusForbidden, "forbidden")
	}
	return nil
}

func requireProjectWritePrincipal(c *fiber.Ctx, authRuntime *platformauth.Runtime, project projectView) (platformauth.Principal, error) {
	principal, ok, err := authRuntime.AuthenticateHeader(c.UserContext(), c.Get(fiber.HeaderAuthorization))
	if err != nil {
		return platformauth.Principal{}, fiber.NewError(http.StatusUnauthorized, "invalid credentials")
	}
	if !ok {
		return platformauth.Principal{}, fiber.NewError(http.StatusUnauthorized, "authentication required")
	}
	scope := platformauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
	allowed, err := authRuntime.CanWriteProject(c.UserContext(), principal, scope)
	if err != nil {
		return platformauth.Principal{}, fiber.NewError(http.StatusForbidden, "authorization failed")
	}
	if !allowed {
		return platformauth.Principal{}, fiber.NewError(http.StatusForbidden, "forbidden")
	}
	return principal, nil
}

func loadProject(ctx context.Context, repo *projectrepo.Repository, rawRepoPath string) (projectView, error) {
	fullPath, err := normalizeRepoFullPath(rawRepoPath)
	if err != nil {
		return projectView{}, fiber.ErrNotFound
	}
	project, err := repo.GetByFullPath(ctx, fullPath)
	if err != nil {
		return projectView{}, fiber.ErrNotFound
	}
	return projectView{ID: project.ID, NamespaceID: project.NamespaceID, FullPath: project.FullPath, Visibility: project.Visibility}, nil
}

type projectView struct {
	ID          int64
	NamespaceID int64
	FullPath    string
	Visibility  string
}

func normalizeRepoFullPath(value string) (string, error) {
	normalized := normalizeRequestPath(value)
	if normalized == "" || !strings.HasSuffix(normalized, ".git") {
		return "", fmt.Errorf("invalid repo path")
	}
	normalized = strings.TrimSuffix(normalized, ".git")
	if normalized == "" || strings.Contains(normalized, "..") {
		return "", fmt.Errorf("invalid repo path")
	}
	return normalized, nil
}

func normalizeRequestPath(value string) string {
	return strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
}

func parseLimit(value string) (int, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return 0, nil
	}
	parsed, err := strconv.Atoi(trimmed)
	if err != nil || parsed < 0 {
		return 0, fmt.Errorf("invalid limit")
	}
	return parsed, nil
}

func isLFSPath(path string, method string) bool {
	normalized := normalizeRequestPath(path)
	if normalized == "" || !strings.Contains(normalized, ".git/") {
		return false
	}
	switch method {
	case fiber.MethodPost:
		return strings.HasSuffix(normalized, "/info/lfs/objects/batch") || strings.HasSuffix(normalized, "/info/lfs/locks") || strings.HasSuffix(normalized, "/info/lfs/locks/verify") || strings.HasSuffix(normalized, "/unlock")
	case fiber.MethodPut, fiber.MethodGet:
		return strings.Contains(normalized, "/info/lfs/objects/") || strings.HasSuffix(normalized, "/info/lfs/locks")
	default:
		return false
	}
}
