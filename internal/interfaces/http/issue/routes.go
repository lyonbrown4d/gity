package issue

import (
	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
)

type Endpoint struct {
	service     *issueservice.Service
	authRuntime *infraauth.Runtime
	mapper      *mapper.Mapper
}

func NewEndpoint(service *issueservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Issues", "Issues", "Project issue APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/issues", e.listIssues),
		httpapi.Get("/repos/{id}/issues", e.listIssues, httpapi.DeprecatedRoute[projectIssuesInput, issueOutput]("Use GET /projects/{id}/issues instead.")),
		httpapi.Get("/projects/{id}/issues/{issue_iid}", e.getIssue),
		httpapi.Get("/repos/{id}/issues/{issue_iid}", e.getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Get("/repos/{id}/issues/by-number/{issue_iid}", e.getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Post("/projects/{id}/issues", e.createIssue, httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues", e.createIssue,
			httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createIssueInput, issueOutput]("Use POST /projects/{id}/issues instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}", e.updateIssue, httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}", e.updateIssue,
			httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[updateIssueInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/comments", e.listComments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/comments", e.listComments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/comments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/comments", e.createComment, httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/comments", e.createComment,
			httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createCommentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/comments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments", e.listAttachments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments", e.listAttachments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/attachments", e.createAttachment, httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/attachments", e.createAttachment,
			httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createAttachmentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/attachments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}", e.getAttachment),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments/{attachment_id}", e.getAttachment, httpapi.DeprecatedRoute[projectAttachmentInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments/{attachment_id} instead.")),
	)
}
