package lfs

import (
	"context"
	"net/http"
	"strings"

	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	"github.com/gofiber/fiber/v2"
	"github.com/samber/oops"
)

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

func authorizeLFSOperation(c *fiber.Ctx, authRuntime *infraauth.Runtime, project projectView, operation string) error {
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

	scope := infraauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
	var allowed bool
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

func requireProjectWritePrincipal(c *fiber.Ctx, authRuntime *infraauth.Runtime, project projectView) (infraauth.Principal, error) {
	principal, ok, err := authRuntime.AuthenticateHeader(c.UserContext(), c.Get(fiber.HeaderAuthorization))
	if err != nil {
		return infraauth.Principal{}, fiber.NewError(http.StatusUnauthorized, "invalid credentials")
	}
	if !ok {
		return infraauth.Principal{}, fiber.NewError(http.StatusUnauthorized, "authentication required")
	}
	scope := infraauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
	allowed, err := authRuntime.CanWriteProject(c.UserContext(), principal, scope)
	if err != nil {
		return infraauth.Principal{}, fiber.NewError(http.StatusForbidden, "authorization failed")
	}
	if !allowed {
		return infraauth.Principal{}, fiber.NewError(http.StatusForbidden, "forbidden")
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
		return "", oops.In("http.lfs").With("path", value).New("invalid repo path")
	}
	normalized = strings.TrimSuffix(normalized, ".git")
	if normalized == "" || strings.Contains(normalized, "..") {
		return "", oops.In("http.lfs").With("path", value, "normalized", normalized).New("invalid repo path")
	}
	return normalized, nil
}
