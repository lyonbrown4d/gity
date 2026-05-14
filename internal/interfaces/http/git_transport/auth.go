package gittransport

import (
	"context"
	"net/http"
	"strings"

	"github.com/gofiber/fiber/v2"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	"github.com/samber/oops"
)

func authorizeProject(c *fiber.Ctx, authRuntime *infraauth.Runtime, project projectView, service string) error {
	if service == serviceUploadPack && strings.TrimSpace(project.Visibility) == "public" {
		return nil
	}
	principal, ok, err := authRuntime.AuthenticateHeader(c.UserContext(), c.Get(fiber.HeaderAuthorization))
	if err != nil {
		return fiber.NewError(http.StatusUnauthorized, "invalid credentials")
	}
	if !ok {
		return fiber.NewError(http.StatusUnauthorized, "authentication required")
	}
	allowed := false
	scope := infraauth.ProjectScope{ID: project.ID, OrganizationID: project.OrganizationID, Visibility: project.Visibility}
	switch service {
	case serviceUploadPack:
		allowed, err = authRuntime.CanProjectAction(c.UserContext(), principal, scope, infraauth.ProjectActionRepositoryRead)
	case serviceReceivePack:
		allowed, err = authRuntime.CanProjectAction(c.UserContext(), principal, scope, infraauth.ProjectActionRepositoryPush)
	}
	if err != nil {
		return fiber.NewError(http.StatusForbidden, "authorization failed")
	}
	if !allowed {
		return fiber.NewError(http.StatusForbidden, "forbidden")
	}
	return nil
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
	return projectView{ID: project.ID, OrganizationID: project.OrganizationID, FullPath: project.FullPath, Visibility: project.Visibility, DefaultBranch: project.DefaultBranch}, nil
}

type projectView struct {
	ID             int64
	OrganizationID int64
	FullPath       string
	Visibility     string
	DefaultBranch  string
}

func normalizeRepoFullPath(value string) (string, error) {
	normalized := strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
	if normalized == "" || !strings.HasSuffix(normalized, ".git") {
		return "", oops.In("http.git_transport").With("path", value).New("invalid repo path")
	}
	normalized = strings.TrimSuffix(normalized, ".git")
	if normalized == "" || strings.Contains(normalized, "..") {
		return "", oops.In("http.git_transport").With("path", value, "normalized", normalized).New("invalid repo path")
	}
	return normalized, nil
}
