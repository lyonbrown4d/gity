package project

import (
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	"github.com/DaiYuANg/gity/internal/config"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type Endpoint struct {
	service         *projectservice.Service
	settings        config.Settings
	authRuntime     *infraauth.Runtime
	pipelineService *pipelineservice.Service
}

func NewEndpoint(service *projectservice.Service, settings config.Settings, authRuntime *infraauth.Runtime, pipelineService *pipelineservice.Service) *Endpoint {
	return &Endpoint{service: service, settings: settings, authRuntime: authRuntime, pipelineService: pipelineService}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Projects", "Projects", "Project and repository APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	projectWrite := e.projectScope

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects", e.listProjects),
		httpapi.Get("/repos", e.listProjects, httpapi.DeprecatedRoute[projectsInput, projectOutput]("Use GET /projects instead.")),
		httpapi.Get("/projects/{id}", e.getProject),
		httpapi.Get("/repos/{id}", e.getProject, httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id} instead.")),
		httpapi.Post("/projects", e.createProject, httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime)),
		httpapi.Post("/repos", e.createProject,
			httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime),
			httpapi.DeprecatedRoute[createProjectInput, projectOutput]("Use POST /projects instead."),
		),
		httpapi.Delete("/projects/{id}", e.deleteProject, httpapi.RequireProjectWriteRoute[deleteProjectInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}", e.deleteProject,
			httpapi.RequireProjectWriteRoute[deleteProjectInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[deleteProjectInput, projectOutput]("Use DELETE /projects/{id} instead."),
		),
		httpapi.Get("/projects/{id}/repository/branches", e.listBranches),
		httpapi.Get("/repos/{id}/branches", e.listBranches, httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id}/repository/branches instead.")),
		httpapi.Post("/projects/{id}/repository/branches", e.createBranch, httpapi.RequireProjectWriteRoute[createBranchInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/branches", e.createBranch,
			httpapi.RequireProjectWriteRoute[createBranchInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createBranchInput, projectOutput]("Use POST /projects/{id}/repository/branches instead."),
		),
		httpapi.Delete("/projects/{id}/repository/branches/{branch_name}", e.deleteBranch, httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}/branches/{branch_name}", e.deleteBranch,
			httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use DELETE /projects/{id}/repository/branches/{branch_name} instead."),
		),
		httpapi.Get("/projects/{id}/repository/branch-protections", e.listBranchProtections),
		httpapi.Patch("/projects/{id}/repository/branch-protections/{branch_name}", e.upsertBranchProtection, httpapi.RequireProjectWriteRoute[upsertBranchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/protect", e.protectBranch, httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/unprotect", e.unprotectBranch, httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/branches/{branch_name}/protect", e.protectBranch,
			httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/protect instead."),
		),
		httpapi.Post("/repos/{id}/branches/{branch_name}/unprotect", e.unprotectBranch,
			httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/unprotect instead."),
		),
		httpapi.Get("/projects/{id}/repository/commits", e.listCommits),
		httpapi.Get("/repos/{id}/commits", e.listCommits, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/commits instead.")),
		httpapi.Get("/projects/{id}/repository/tree", e.listTree),
		httpapi.Get("/repos/{id}/tree", e.listTree, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/tree instead.")),
		httpapi.Get("/projects/{id}/repository/blob", e.getBlob),
		httpapi.Get("/repos/{id}/blob", e.getBlob, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/blob instead.")),
		httpapi.Get("/projects/{id}/repository/readme", e.getReadme),
		httpapi.Get("/repos/{id}/readme", e.getReadme, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/readme instead.")),
		httpapi.Get("/projects/{id}/repository/search", e.searchRepository),
		httpapi.Get("/repos/{id}/search", e.searchRepository, httpapi.DeprecatedRoute[projectRepositorySearchInput, projectOutput]("Use GET /projects/{id}/repository/search instead.")),
		httpapi.Post("/projects/{id}/repository/files", e.createFileCommit, httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/projects/{id}/file-commits", e.createFileCommit,
			httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Post("/repos/{id}/file-commits", e.createFileCommit,
			httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Get("/projects/{id}/languages", e.languages),
		httpapi.Get("/repos/{id}/languages", e.languages, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/languages instead.")),
	)
}
