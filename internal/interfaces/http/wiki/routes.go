package wiki

import (
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	wikiservice "github.com/DaiYuANg/gity/internal/application/wiki"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
)

type wikiPagesInput struct {
	ProjectID int64 `path:"id"`
}

type wikiPageInput struct {
	ProjectID     int64  `path:"id"`
	Slug          string `path:"slug"`
	Authorization string `header:"Authorization"`
}

type createWikiPageInput struct {
	ProjectID     int64              `path:"id"`
	Authorization string             `header:"Authorization"`
	Body          createWikiPageBody `json:"body"`
}

type updateWikiPageInput struct {
	ProjectID     int64              `path:"id"`
	Slug          string             `path:"slug"`
	Authorization string             `header:"Authorization"`
	Body          updateWikiPageBody `json:"body"`
}

type createWikiPageBody struct {
	Slug         string `json:"slug"`
	Title        string `json:"title"`
	Content      string `json:"content"`
	Format       string `json:"format"`
	AuthorUserID int64  `json:"author_user_id"`
}

type updateWikiPageBody struct {
	Title        *string `json:"title"`
	Content      *string `json:"content"`
	EditorUserID int64   `json:"editor_user_id"`
}

type wikiOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *wikiservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *wikiservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Wiki", "Wiki", "Project wiki APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/wiki/pages", e.listPages),
		httpapi.Get("/repos/{id}/wiki/pages", e.listPages, httpapi.DeprecatedRoute[wikiPagesInput, wikiOutput]("Use GET /projects/{id}/wiki/pages instead.")),
		httpapi.Post("/projects/{id}/wiki/pages", e.createPage, httpapi.RequireProjectWriteRoute[createWikiPageInput, wikiOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/wiki/pages", e.createPage,
			httpapi.RequireProjectWriteRoute[createWikiPageInput, wikiOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createWikiPageInput, wikiOutput]("Use POST /projects/{id}/wiki/pages instead."),
		),
		httpapi.Get("/projects/{id}/wiki/pages/{slug}", e.getPage),
		httpapi.Get("/repos/{id}/wiki/pages/{slug}", e.getPage, httpapi.DeprecatedRoute[wikiPageInput, wikiOutput]("Use GET /projects/{id}/wiki/pages/{slug} instead.")),
		httpapi.Patch("/projects/{id}/wiki/pages/{slug}", e.updatePage, httpapi.RequireProjectWriteRoute[updateWikiPageInput, wikiOutput](authRuntime, projectWrite)),
		httpapi.Patch("/repos/{id}/wiki/pages/{slug}", e.updatePage,
			httpapi.RequireProjectWriteRoute[updateWikiPageInput, wikiOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[updateWikiPageInput, wikiOutput]("Use PATCH /projects/{id}/wiki/pages/{slug} instead."),
		),
		httpapi.Delete("/projects/{id}/wiki/pages/{slug}", e.deletePage, httpapi.RequireProjectWriteRoute[wikiPageInput, wikiOutput](authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}/wiki/pages/{slug}", e.deletePage,
			httpapi.RequireProjectWriteRoute[wikiPageInput, wikiOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[wikiPageInput, wikiOutput]("Use DELETE /projects/{id}/wiki/pages/{slug} instead."),
		),
	)
}

func (in wikiPageInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in wikiPageInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createWikiPageInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createWikiPageInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateWikiPageInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateWikiPageInput) ProjectIDValue() int64 {
	return in.ProjectID
}
