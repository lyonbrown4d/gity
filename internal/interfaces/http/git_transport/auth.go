package gittransport

import (
	"context"
	"net/http"
	"strings"

	"github.com/gofiber/fiber/v3"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	"github.com/samber/oops"
)

func authorizeProject(c fiber.Ctx, authRuntime *infraauth.Runtime, project projectView, service string) (infraauth.Principal, error) {
	if service == serviceUploadPack && strings.TrimSpace(project.Visibility) == "public" {
		return infraauth.Principal{}, nil
	}
	principal, ok, err := authRuntime.AuthenticateHeader(c.Context(), c.Get(fiber.HeaderAuthorization))
	if err != nil {
		return infraauth.Principal{}, gitAuthChallenge(c, "invalid credentials")
	}
	if !ok {
		return infraauth.Principal{}, gitAuthChallenge(c, "authentication required")
	}
	allowed := false
	scope := infraauth.ProjectScope{ID: project.ID, OrganizationID: project.OrganizationID, Visibility: project.Visibility}
	switch service {
	case serviceUploadPack:
		allowed, err = authRuntime.CanProjectAction(c.Context(), principal, scope, infraauth.ProjectActionRepositoryRead)
	case serviceReceivePack:
		allowed, err = authRuntime.CanProjectAction(c.Context(), principal, scope, infraauth.ProjectActionRepositoryPush)
	}
	if err != nil {
		return infraauth.Principal{}, fiber.NewError(http.StatusForbidden, "authorization failed")
	}
	if !allowed {
		return infraauth.Principal{}, fiber.NewError(http.StatusForbidden, "forbidden")
	}
	return principal, nil
}

func gitAuthChallenge(c fiber.Ctx, message string) error {
	// Git clients only retry URL credentials after a standard Basic challenge.
	c.Set(fiber.HeaderWWWAuthenticate, `Basic realm="Gity Git"`)
	return fiber.NewError(http.StatusUnauthorized, message)
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
