package gittransport

import (
	"bytes"
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	platformgittransport "github.com/DaiYuANg/gity/internal/platform/gittransport"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	"github.com/gofiber/fiber/v2"
)

const (
	serviceUploadPack  = "git-upload-pack"
	serviceReceivePack = "git-receive-pack"
)

func RegisterRoutes(app *fiber.App, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, transport *platformgittransport.Service) {
	app.Use(func(c *fiber.Ctx) error {
		if !isGitProtocolPath(c.Path(), c.Method()) {
			return c.Next()
		}
		switch c.Method() {
		case fiber.MethodGet:
			return handleInfoRefs(c, logger, authRuntime, repo, transport)
		case fiber.MethodPost:
			return handleRPC(c, logger, authRuntime, repo, transport)
		default:
			return c.Next()
		}
	})
}

func handleInfoRefs(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, transport *platformgittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	if !strings.HasSuffix(path, "/info/refs") {
		return c.Next()
	}
	service := strings.TrimSpace(c.Query("service"))
	if service != serviceUploadPack && service != serviceReceivePack {
		return fiber.NewError(http.StatusBadRequest, "unsupported git service")
	}
	repoPath := strings.TrimSuffix(path, "/info/refs")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	if err := authorizeProject(c, authRuntime, project, service); err != nil {
		return err
	}

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	switch service {
	case serviceUploadPack:
		err = transport.AdvertiseUploadPack(c.UserContext(), project.FullPath+".git", &stdout, &stderr)
	case serviceReceivePack:
		err = transport.AdvertiseReceivePack(c.UserContext(), project.FullPath+".git", &stdout, &stderr)
	}
	if err != nil {
		logger.Error("git advertise failed", slog.String("project", project.FullPath), slog.String("service", service), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git advertise failed")
	}

	contentType := map[string]string{
		serviceUploadPack:  "application/x-git-upload-pack-advertisement",
		serviceReceivePack: "application/x-git-receive-pack-advertisement",
	}[service]
	body := append([]byte(pktLine("# service="+service+"\n")+"0000"), stdout.Bytes()...)
	c.Set(fiber.HeaderContentType, contentType)
	c.Set(fiber.HeaderCacheControl, "no-cache")
	return c.Send(body)
}

func handleRPC(c *fiber.Ctx, logger *slog.Logger, authRuntime *platformauth.Runtime, repo *projectrepo.Repository, transport *platformgittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	service := ""
	repoPath := ""
	switch {
	case strings.HasSuffix(path, "/git-upload-pack"):
		service = serviceUploadPack
		repoPath = strings.TrimSuffix(path, "/git-upload-pack")
	case strings.HasSuffix(path, "/git-receive-pack"):
		service = serviceReceivePack
		repoPath = strings.TrimSuffix(path, "/git-receive-pack")
	default:
		return c.Next()
	}
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	if err := authorizeProject(c, authRuntime, project, service); err != nil {
		return err
	}

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	switch service {
	case serviceUploadPack:
		err = transport.UploadPack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), &stdout, &stderr)
	case serviceReceivePack:
		err = transport.ReceivePack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), &stdout, &stderr)
	}
	if err != nil {
		logger.Error("git rpc failed", slog.String("project", project.FullPath), slog.String("service", service), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git rpc failed")
	}

	contentType := map[string]string{
		serviceUploadPack:  "application/x-git-upload-pack-result",
		serviceReceivePack: "application/x-git-receive-pack-result",
	}[service]
	c.Set(fiber.HeaderContentType, contentType)
	c.Set(fiber.HeaderCacheControl, "no-cache")
	return c.Send(stdout.Bytes())
}

func authorizeProject(c *fiber.Ctx, authRuntime *platformauth.Runtime, project projectView, service string) error {
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
	scope := platformauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
	switch service {
	case serviceUploadPack:
		allowed, err = authRuntime.CanReadProject(c.UserContext(), principal, scope)
	case serviceReceivePack:
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
	normalized := strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
	if normalized == "" || !strings.HasSuffix(normalized, ".git") {
		return "", fmt.Errorf("invalid repo path")
	}
	normalized = strings.TrimSuffix(normalized, ".git")
	if normalized == "" || strings.Contains(normalized, "..") {
		return "", fmt.Errorf("invalid repo path")
	}
	return normalized, nil
}

func pktLine(data string) string {
	total := len(data) + 4
	return fmt.Sprintf("%04x%s", total, data)
}

func isGitProtocolPath(path string, method string) bool {
	normalized := strings.Trim(strings.ReplaceAll(path, "\\", "/"), "/")
	if normalized == "" || !strings.Contains(normalized, ".git/") {
		return false
	}
	switch method {
	case fiber.MethodGet:
		return strings.HasSuffix(normalized, "/info/refs")
	case fiber.MethodPost:
		return strings.HasSuffix(normalized, "/git-upload-pack") || strings.HasSuffix(normalized, "/git-receive-pack")
	default:
		return false
	}
}
