package gittransport

import (
	"bytes"
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	"github.com/DaiYuANg/gity/internal/platform/gittransport"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	"github.com/gofiber/fiber/v2"
)

const (
	serviceUploadPack = "git-upload-pack"
)

func RegisterRoutes(app *fiber.App, logger *slog.Logger, repo *projectrepo.Repository, transport *gittransport.Service) {
	app.Use(func(c *fiber.Ctx) error {
		if !isGitProtocolPath(c.Path(), c.Method()) {
			return c.Next()
		}
		switch c.Method() {
		case fiber.MethodGet:
			return handleInfoRefs(c, logger, repo, transport)
		case fiber.MethodPost:
			return handleRPC(c, logger, repo, transport)
		default:
			return c.Next()
		}
	})
}

func handleInfoRefs(c *fiber.Ctx, logger *slog.Logger, repo *projectrepo.Repository, transport *gittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	if !strings.HasSuffix(path, "/info/refs") {
		return c.Next()
	}
	service := strings.TrimSpace(c.Query("service"))
	if service != serviceUploadPack {
		return fiber.NewError(http.StatusBadRequest, "unsupported git service")
	}
	repoPath := strings.TrimSuffix(path, "/info/refs")
	project, err := loadPublicProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	if err := transport.AdvertiseUploadPack(c.UserContext(), project.FullPath+".git", &stdout, &stderr); err != nil {
		logger.Error("git upload-pack advertise failed", slog.String("project", project.FullPath), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git upload-pack advertise failed")
	}

	body := append([]byte(pktLine("# service="+serviceUploadPack+"\n")+"0000"), stdout.Bytes()...)
	c.Set(fiber.HeaderContentType, "application/x-git-upload-pack-advertisement")
	c.Set(fiber.HeaderCacheControl, "no-cache")
	return c.Send(body)
}

func handleRPC(c *fiber.Ctx, logger *slog.Logger, repo *projectrepo.Repository, transport *gittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	if !strings.HasSuffix(path, "/git-upload-pack") {
		return c.Next()
	}
	repoPath := strings.TrimSuffix(path, "/git-upload-pack")
	project, err := loadPublicProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	if err := transport.UploadPack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), &stdout, &stderr); err != nil {
		logger.Error("git upload-pack failed", slog.String("project", project.FullPath), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git upload-pack failed")
	}

	c.Set(fiber.HeaderContentType, "application/x-git-upload-pack-result")
	c.Set(fiber.HeaderCacheControl, "no-cache")
	return c.Send(stdout.Bytes())
}

func loadPublicProject(ctx context.Context, repo *projectrepo.Repository, rawRepoPath string) (projectrepoProject, error) {
	fullPath, err := normalizeRepoFullPath(rawRepoPath)
	if err != nil {
		return projectrepoProject{}, fiber.ErrNotFound
	}
	project, err := repo.GetByFullPath(ctx, fullPath)
	if err != nil {
		return projectrepoProject{}, fiber.ErrNotFound
	}
	if strings.TrimSpace(project.Visibility) != "public" {
		return projectrepoProject{}, fiber.ErrNotFound
	}
	return projectrepoProject{FullPath: project.FullPath, Visibility: project.Visibility}, nil
}

type projectrepoProject struct {
	FullPath   string
	Visibility string
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
