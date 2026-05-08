package gittransport

import (
	"bytes"
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	infragittransport "github.com/DaiYuANg/gity/internal/infrastructure/git_transport"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	collectionlist "github.com/arcgolabs/collectionx/list"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	setx "github.com/arcgolabs/collectionx/set"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/gofiber/fiber/v2"
	"github.com/samber/oops"
)

const (
	serviceUploadPack  = "git-upload-pack"
	serviceReceivePack = "git-receive-pack"
)

var (
	supportedGitServices      = setx.NewSet(serviceUploadPack, serviceReceivePack)
	advertisementContentTypes = mappingx.NewMapFrom(map[string]string{
		serviceUploadPack:  "application/x-git-upload-pack-advertisement",
		serviceReceivePack: "application/x-git-receive-pack-advertisement",
	})
	rpcContentTypes = mappingx.NewMapFrom(map[string]string{
		serviceUploadPack:  "application/x-git-upload-pack-result",
		serviceReceivePack: "application/x-git-receive-pack-result",
	})
)

type RouteServices struct {
	branchProtectionRepo *projectbranchprotectionrepo.Repository
	pipelineService      *pipelineservice.Service
}

func NewRouteServices(branchProtectionRepo *projectbranchprotectionrepo.Repository, pipelineService *pipelineservice.Service) *RouteServices {
	return &RouteServices{branchProtectionRepo: branchProtectionRepo, pipelineService: pipelineService}
}

func RegisterRoutes(app *fiber.App, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, transport *infragittransport.Service, services *RouteServices) {
	app.Use(func(c *fiber.Ctx) error {
		if !isGitProtocolPath(c.Path(), c.Method()) {
			return c.Next()
		}
		switch c.Method() {
		case fiber.MethodGet:
			return handleInfoRefs(c, logger, authRuntime, repo, transport)
		case fiber.MethodPost:
			return handleRPC(c, logger, authRuntime, repo, services, transport)
		default:
			return c.Next()
		}
	})
}

func handleInfoRefs(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, transport *infragittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	if !strings.HasSuffix(path, "/info/refs") {
		return c.Next()
	}
	service := strings.TrimSpace(c.Query("service"))
	if !supportedGitServices.Contains(service) {
		return fiber.NewError(http.StatusBadRequest, "unsupported git service")
	}
	repoPath := strings.TrimSuffix(path, "/info/refs")
	project, err := loadProject(c.UserContext(), repo, repoPath)
	if err != nil {
		return err
	}
	if authErr := authorizeProject(c, authRuntime, project, service); authErr != nil {
		return authErr
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

	contentType, _ := advertisementContentTypes.Get(service)
	body := append([]byte(pktLine("# service="+service+"\n")+"0000"), stdout.Bytes()...)
	c.Set(fiber.HeaderContentType, contentType)
	c.Set(fiber.HeaderCacheControl, "no-cache")
	return c.Send(body)
}

func handleRPC(c *fiber.Ctx, logger *slog.Logger, authRuntime *infraauth.Runtime, repo *projectrepo.Repository, services *RouteServices, transport *infragittransport.Service) error {
	path := strings.Trim(strings.ReplaceAll(c.Path(), "\\", "/"), "/")
	var service string
	var repoPath string
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
	if authErr := authorizeProject(c, authRuntime, project, service); authErr != nil {
		return authErr
	}

	updates := parseReceivePackUpdates(c.Body())
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	switch service {
	case serviceUploadPack:
		err = transport.UploadPack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), &stdout, &stderr)
	case serviceReceivePack:
		if protectionErr := rejectProtectedBranchUpdates(c.UserContext(), project, services.branchProtectionRepo, updates); protectionErr != nil {
			return protectionErr
		}
		err = transport.ReceivePack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), &stdout, &stderr)
	}
	if err != nil {
		logger.Error("git rpc failed", slog.String("project", project.FullPath), slog.String("service", service), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git rpc failed")
	}

	contentType, _ := rpcContentTypes.Get(service)
	c.Set(fiber.HeaderContentType, contentType)
	c.Set(fiber.HeaderCacheControl, "no-cache")
	if service == serviceReceivePack {
		triggerPushPipelines(c.UserContext(), logger, services.pipelineService, project, updates)
	}
	return c.Send(stdout.Bytes())
}

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
	scope := infraauth.ProjectScope{ID: project.ID, NamespaceID: project.NamespaceID, Visibility: project.Visibility}
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
		return "", oops.In("http.git_transport").With("path", value).New("invalid repo path")
	}
	normalized = strings.TrimSuffix(normalized, ".git")
	if normalized == "" || strings.Contains(normalized, "..") {
		return "", oops.In("http.git_transport").With("path", value, "normalized", normalized).New("invalid repo path")
	}
	return normalized, nil
}

func pktLine(data string) string {
	total := len(data) + 4
	return fmt.Sprintf("%04x%s", total, data)
}

type receivePackUpdate struct {
	OldSHA     string
	NewSHA     string
	RefName    string
	BranchName string
	Delete     bool
}

func parseReceivePackUpdates(body []byte) []receivePackUpdate {
	updates := collectionlist.NewList[receivePackUpdate]()
	offset := 0
	for offset+4 <= len(body) {
		rawLength := string(body[offset : offset+4])
		offset += 4
		if rawLength == "0000" {
			break
		}
		lengthBytes, err := hex.DecodeString(rawLength)
		if err != nil || len(lengthBytes) != 2 {
			break
		}
		length := int(lengthBytes[0])<<8 + int(lengthBytes[1])
		if length < 4 {
			break
		}
		payloadLength := length - 4
		if offset+payloadLength > len(body) {
			break
		}
		payload := string(body[offset : offset+payloadLength])
		offset += payloadLength
		if index := strings.IndexByte(payload, 0); index >= 0 {
			payload = payload[:index]
		}
		fields := strings.Fields(payload)
		if len(fields) < 3 {
			continue
		}
		refName := fields[2]
		if strings.HasPrefix(refName, "refs/heads/") {
			newSHA := fields[1]
			updates.Add(receivePackUpdate{
				OldSHA:     fields[0],
				NewSHA:     newSHA,
				RefName:    refName,
				BranchName: strings.TrimPrefix(refName, "refs/heads/"),
				Delete:     isZeroOID(newSHA),
			})
		}
	}
	return updates.Values()
}

func triggerPushPipelines(ctx context.Context, logger *slog.Logger, service *pipelineservice.Service, project projectView, updates []receivePackUpdate) {
	if service == nil {
		return
	}
	for _, update := range updates {
		if update.BranchName == "" || update.Delete || isZeroOID(update.NewSHA) {
			continue
		}
		view, created, err := service.CreatePushPipeline(ctx, project.ID, update.BranchName, update.NewSHA)
		if err != nil {
			logger.Warn("create push pipeline failed", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.String("error", err.Error()))
			continue
		}
		if view.Pipeline.ID == 0 {
			continue
		}
		if created {
			logger.Info("push pipeline created", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.Int64("pipeline_id", view.Pipeline.ID))
		} else {
			logger.Debug("push pipeline already exists", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.Int64("pipeline_id", view.Pipeline.ID))
		}
	}
}

func isZeroOID(value string) bool {
	value = strings.TrimSpace(value)
	return value == "" || value == "0000000000000000000000000000000000000000"
}

func isGitProtocolPath(path, method string) bool {
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
