package project

import (
	"github.com/arcgolabs/httpx"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

func (e *Endpoint) registerProjectRoutes(registrar httpx.Registrar, projectScope httpapi.ProjectScopeResolver) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects", e.listProjects, httpapi.RequireUserRoute[projectsInput, projectOutput](e.authRuntime)),
		httpapi.Get("/repos", e.listProjects,
			httpapi.RequireUserRoute[projectsInput, projectOutput](e.authRuntime),
			httpapi.DeprecatedRoute[projectsInput, projectOutput]("Use GET /projects instead."),
		),
		httpapi.Get("/projects/{id}", e.getProject, httpapi.RequireProjectReadRoute[projectByIDInput, projectOutput](e.authRuntime, projectScope)),
		httpapi.Get("/repos/{id}", e.getProject,
			httpapi.RequireProjectReadRoute[projectByIDInput, projectOutput](e.authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id} instead."),
		),
		httpapi.Get("/projects/{id}/permissions", e.getPermissions, httpapi.RequireUserRoute[projectByIDInput, projectOutput](e.authRuntime)),
		httpapi.Get("/repos/{id}/permissions", e.getPermissions,
			httpapi.RequireUserRoute[projectByIDInput, projectOutput](e.authRuntime),
			httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id}/permissions instead."),
		),
		httpapi.Get("/projects/{id}/members", e.listMembers, httpapi.RequireProjectActionRoute[projectByIDInput, projectOutput]("require_project_members_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)),
		httpapi.Post("/projects/{id}/members", e.createMember, httpapi.RequireProjectActionRoute[createProjectMemberInput, projectOutput]("require_project_members_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Patch("/projects/{id}/members/{user_id}", e.upsertMember, httpapi.RequireProjectActionRoute[upsertProjectMemberInput, projectOutput]("require_project_members_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Delete("/projects/{id}/members/{user_id}", e.deleteMember, httpapi.RequireProjectActionRoute[projectMemberInput, projectOutput]("require_project_members_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Post("/projects", e.createProject, httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime)),
		httpapi.Post("/repos", e.createProject,
			httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime),
			httpapi.DeprecatedRoute[createProjectInput, projectOutput]("Use POST /projects instead."),
		),
		httpapi.Delete("/projects/{id}", e.deleteProject, httpapi.RequireProjectActionRoute[deleteProjectInput, projectOutput]("require_project_delete", e.authRuntime, projectScope, infraauth.ProjectActionDelete)),
		httpapi.Delete("/repos/{id}", e.deleteProject,
			httpapi.RequireProjectActionRoute[deleteProjectInput, projectOutput]("require_project_delete", e.authRuntime, projectScope, infraauth.ProjectActionDelete),
			httpapi.DeprecatedRoute[deleteProjectInput, projectOutput]("Use DELETE /projects/{id} instead."),
		),
	)
}

func (e *Endpoint) registerBranchRoutes(registrar httpx.Registrar, projectScope httpapi.ProjectScopeResolver) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/repository/branches", e.listBranches, httpapi.RequireProjectActionRoute[projectByIDInput, projectOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)),
		httpapi.Get("/repos/{id}/branches", e.listBranches,
			httpapi.RequireProjectActionRoute[projectByIDInput, projectOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead),
			httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id}/repository/branches instead."),
		),
		httpapi.Post("/projects/{id}/repository/branches", e.createBranch, httpapi.RequireProjectActionRoute[createBranchInput, projectOutput]("require_repository_push", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryPush)),
		httpapi.Post("/repos/{id}/branches", e.createBranch,
			httpapi.RequireProjectActionRoute[createBranchInput, projectOutput]("require_repository_push", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryPush),
			httpapi.DeprecatedRoute[createBranchInput, projectOutput]("Use POST /projects/{id}/repository/branches instead."),
		),
		httpapi.Delete("/projects/{id}/repository/branches/{branch_name}", e.deleteBranch, httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Delete("/repos/{id}/branches/{branch_name}", e.deleteBranch,
			httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use DELETE /projects/{id}/repository/branches/{branch_name} instead."),
		),
		httpapi.Get("/projects/{id}/repository/branch-protections", e.listBranchProtections, httpapi.RequireProjectActionRoute[projectByIDInput, projectOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)),
		httpapi.Patch("/projects/{id}/repository/branch-protections/{branch_name}", e.upsertBranchProtection, httpapi.RequireProjectActionRoute[upsertBranchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/protect", e.protectBranch, httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/unprotect", e.unprotectBranch, httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Post("/repos/{id}/branches/{branch_name}/protect", e.protectBranch,
			httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/protect instead."),
		),
		httpapi.Post("/repos/{id}/branches/{branch_name}/unprotect", e.unprotectBranch,
			httpapi.RequireProjectActionRoute[branchProtectionInput, projectOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/unprotect instead."),
		),
	)
}

func (e *Endpoint) registerRepositoryRoutes(registrar httpx.Registrar, projectScope httpapi.ProjectScopeResolver) {
	repositoryRead := httpapi.RequireProjectActionRoute[projectRepositoryInput, projectOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)
	repositorySearchRead := httpapi.RequireProjectActionRoute[projectRepositorySearchInput, projectOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)
	repositoryPush := httpapi.RequireProjectActionRoute[createFileCommitInput, projectOutput]("require_repository_push", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryPush)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/repository/commits", e.listCommits, repositoryRead),
		httpapi.Get("/repos/{id}/commits", e.listCommits, repositoryRead, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/commits instead.")),
		httpapi.Get("/projects/{id}/repository/tree", e.listTree, repositoryRead),
		httpapi.Get("/repos/{id}/tree", e.listTree, repositoryRead, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/tree instead.")),
		httpapi.Get("/projects/{id}/repository/blob", e.getBlob, repositoryRead),
		httpapi.Get("/repos/{id}/blob", e.getBlob, repositoryRead, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/blob instead.")),
		httpapi.Get("/projects/{id}/repository/readme", e.getReadme, repositoryRead),
		httpapi.Get("/repos/{id}/readme", e.getReadme, repositoryRead, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/readme instead.")),
		httpapi.Get("/projects/{id}/repository/search", e.searchRepository, repositorySearchRead),
		httpapi.Get("/repos/{id}/search", e.searchRepository, repositorySearchRead, httpapi.DeprecatedRoute[projectRepositorySearchInput, projectOutput]("Use GET /projects/{id}/repository/search instead.")),
		httpapi.Post("/projects/{id}/repository/files", e.createFileCommit, repositoryPush),
		httpapi.Post("/projects/{id}/file-commits", e.createFileCommit,
			repositoryPush,
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Post("/repos/{id}/file-commits", e.createFileCommit,
			repositoryPush,
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Get("/projects/{id}/languages", e.languages, repositoryRead),
		httpapi.Get("/repos/{id}/languages", e.languages, repositoryRead, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/languages instead.")),
		httpapi.Post("/projects/{id}/search/index/refresh", e.refreshSearchIndex,
			httpapi.RequireProjectActionRoute[refreshProjectSearchIndexInput, projectOutput](
				"require_search_index_refresh",
				e.authRuntime,
				projectScope,
				infraauth.ProjectActionRepositoryAdmin,
			),
		),
		httpapi.Post("/repos/{id}/search/index/refresh", e.refreshSearchIndex,
			httpapi.RequireProjectActionRoute[refreshProjectSearchIndexInput, projectOutput](
				"require_search_index_refresh",
				e.authRuntime,
				projectScope,
				infraauth.ProjectActionRepositoryAdmin,
			),
			httpapi.DeprecatedRoute[refreshProjectSearchIndexInput, projectOutput]("Use POST /projects/{id}/search/index/refresh instead."),
		),
	)
}
