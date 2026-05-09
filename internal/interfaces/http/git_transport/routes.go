package gittransport

import (
	"bytes"
	"log/slog"
	"net/http"
	"strings"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	infragittransport "github.com/DaiYuANg/gity/internal/infrastructure/git_transport"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	setx "github.com/arcgolabs/collectionx/set"
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
	rpc, ok := parseRPCPath(path)
	if !ok {
		return c.Next()
	}
	project, err := loadProject(c.UserContext(), repo, rpc.repoPath)
	if err != nil {
		return err
	}
	if authErr := authorizeProject(c, authRuntime, project, rpc.service); authErr != nil {
		return authErr
	}

	updates := ParseReceivePackUpdates(c.Body())
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	if err := executeRPC(c, project, rpc.service, services, transport, updates, &stdout, &stderr); err != nil {
		logger.Error("git rpc failed", slog.String("project", project.FullPath), slog.String("service", rpc.service), slog.String("error", err.Error()), slog.String("stderr", stderr.String()))
		return fiber.NewError(http.StatusInternalServerError, "git rpc failed")
	}
	contentType, _ := rpcContentTypes.Get(rpc.service)
	c.Set(fiber.HeaderContentType, contentType)
	c.Set(fiber.HeaderCacheControl, "no-cache")
	if rpc.service == serviceReceivePack {
		triggerPushPipelines(c.UserContext(), logger, services.pipelineService, project, updates)
	}
	return c.Send(stdout.Bytes())
}

type rpcPath struct {
	service  string
	repoPath string
}

func parseRPCPath(path string) (rpcPath, bool) {
	switch {
	case strings.HasSuffix(path, "/git-upload-pack"):
		return rpcPath{service: serviceUploadPack, repoPath: strings.TrimSuffix(path, "/git-upload-pack")}, true
	case strings.HasSuffix(path, "/git-receive-pack"):
		return rpcPath{service: serviceReceivePack, repoPath: strings.TrimSuffix(path, "/git-receive-pack")}, true
	default:
		return rpcPath{}, false
	}
}

func executeRPC(c *fiber.Ctx, project projectView, service string, services *RouteServices, transport *infragittransport.Service, updates []receivePackUpdate, stdout, stderr *bytes.Buffer) error {
	switch service {
	case serviceUploadPack:
		if err := transport.UploadPack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), stdout, stderr); err != nil {
			return oops.In("http.git_transport").With("project", project.FullPath, "service", service).Wrapf(err, "execute upload-pack")
		}
		return nil
	case serviceReceivePack:
		if err := rejectProtectedBranchUpdates(c.UserContext(), project, services.branchProtectionRepo, updates); err != nil {
			return err
		}
		if err := transport.ReceivePack(c.UserContext(), project.FullPath+".git", bytes.NewReader(c.Body()), stdout, stderr); err != nil {
			return oops.In("http.git_transport").With("project", project.FullPath, "service", service).Wrapf(err, "execute receive-pack")
		}
		return nil
	default:
		return fiber.NewError(http.StatusBadRequest, "unsupported git service")
	}
}
