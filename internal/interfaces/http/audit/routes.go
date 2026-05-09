package audit

import (
	auditservice "github.com/DaiYuANg/gity/internal/application/audit"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type projectAuditEventsInput struct {
	ProjectID     int64  `path:"id"`
	Limit         int    `query:"limit"`
	Authorization string `header:"Authorization"`
}

type auditOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *auditservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
}

func NewEndpoint(service *auditservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Audit", "Audit", "Project audit log APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/audit-events", e.listProjectAuditEvents, httpapi.RequireProjectWriteRoute[projectAuditEventsInput, auditOutput](e.authRuntime, projectWrite)),
	)
}

func (in projectAuditEventsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectAuditEventsInput) ProjectIDValue() int64 {
	return in.ProjectID
}
