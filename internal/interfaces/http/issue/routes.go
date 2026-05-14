package issue

import (
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
	issueservice "github.com/lyonbrown4d/gity/internal/application/issue"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type Endpoint struct {
	service        *issueservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *issueservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Issues", "Issues", "Project issue APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/issues", e.listIssues, httpapi.RequireProjectReadRoute[projectIssuesInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues", e.listIssues,
			httpapi.RequireProjectReadRoute[projectIssuesInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssuesInput, issueOutput]("Use GET /projects/{id}/issues instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}", e.getIssue, httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}", e.getIssue,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Get("/repos/{id}/issues/by-number/{issue_iid}", e.getIssue,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Post("/projects/{id}/issues", e.createIssue, httpapi.RequireProjectActionRoute[createIssueInput, issueOutput]("require_issue_create", authRuntime, projectScope, infraauth.ProjectActionIssueCreate)),
		httpapi.Post("/repos/{id}/issues", e.createIssue,
			httpapi.RequireProjectActionRoute[createIssueInput, issueOutput]("require_issue_create", authRuntime, projectScope, infraauth.ProjectActionIssueCreate),
			httpapi.DeprecatedRoute[createIssueInput, issueOutput]("Use POST /projects/{id}/issues instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}", e.updateIssue, httpapi.RequireProjectActionRoute[updateIssueInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}", e.updateIssue,
			httpapi.RequireProjectActionRoute[updateIssueInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite),
			httpapi.DeprecatedRoute[updateIssueInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/comments", e.listComments, httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/comments", e.listComments,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/comments instead."),
		),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/comments", e.createComment, httpapi.RequireProjectActionRoute[createCommentInput, issueOutput]("require_issue_comment", authRuntime, projectScope, infraauth.ProjectActionIssueComment)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/comments", e.createComment,
			httpapi.RequireProjectActionRoute[createCommentInput, issueOutput]("require_issue_comment", authRuntime, projectScope, infraauth.ProjectActionIssueComment),
			httpapi.DeprecatedRoute[createCommentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/comments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/assignees", e.listAssignees, httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/assignees", e.listAssignees,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/assignees instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}/assignees", e.setAssignees, httpapi.RequireProjectActionRoute[setIssueAssigneesInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}/assignees", e.setAssignees,
			httpapi.RequireProjectActionRoute[setIssueAssigneesInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite),
			httpapi.DeprecatedRoute[setIssueAssigneesInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid}/assignees instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/labels", e.listLabels, httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/labels", e.listLabels,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/labels instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}/labels", e.setLabels, httpapi.RequireProjectActionRoute[setIssueLabelsInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}/labels", e.setLabels,
			httpapi.RequireProjectActionRoute[setIssueLabelsInput, issueOutput]("require_issue_write", authRuntime, projectScope, infraauth.ProjectActionIssueWrite),
			httpapi.DeprecatedRoute[setIssueLabelsInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid}/labels instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments", e.listAttachments, httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments", e.listAttachments,
			httpapi.RequireProjectReadRoute[projectIssueInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments instead."),
		),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/attachments", e.createAttachment, httpapi.RequireProjectActionRoute[createAttachmentInput, issueOutput]("require_issue_comment", authRuntime, projectScope, infraauth.ProjectActionIssueComment)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/attachments", e.createAttachment,
			httpapi.RequireProjectActionRoute[createAttachmentInput, issueOutput]("require_issue_comment", authRuntime, projectScope, infraauth.ProjectActionIssueComment),
			httpapi.DeprecatedRoute[createAttachmentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/attachments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}", e.getAttachment, httpapi.RequireProjectReadRoute[projectAttachmentInput, issueOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments/{attachment_id}", e.getAttachment,
			httpapi.RequireProjectReadRoute[projectAttachmentInput, issueOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[projectAttachmentInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments/{attachment_id} instead."),
		),
	)
}
