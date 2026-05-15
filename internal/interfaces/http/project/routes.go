package project

import (
	"github.com/arcgolabs/httpx"
	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	"github.com/lyonbrown4d/gity/internal/config"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type Endpoint struct {
	service             *projectservice.Service
	memberService       *projectservice.MemberService
	settings            config.Settings
	authRuntime         *infraauth.Runtime
	pipelineService     *pipelineservice.Service
	organizationService *organizationservice.Service
}

func NewEndpoint(service *projectservice.Service, memberService *projectservice.MemberService, settings config.Settings, authRuntime *infraauth.Runtime, pipelineService *pipelineservice.Service, organizationService *organizationservice.Service) *Endpoint {
	return &Endpoint{service: service, memberService: memberService, settings: settings, authRuntime: authRuntime, pipelineService: pipelineService, organizationService: organizationService}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Projects", "Projects", "Project and repository APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	projectScope := httpapi.ProjectScopeResolver(e.projectScope)
	e.registerProjectRoutes(registrar, projectScope)
	e.registerBranchRoutes(registrar, projectScope)
	e.registerRepositoryRoutes(registrar, projectScope)
}
