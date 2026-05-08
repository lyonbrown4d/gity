package issue

import (
	"context"

	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	collectionlist "github.com/arcgolabs/collectionx/list"
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
	service := e.service
	authRuntime := e.authRuntime

	listIssues := func(ctx context.Context, in *projectIssuesInput) (*issueOutput, error) {
		items, err := service.ListIssues(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item issuedomain.ProjectIssue) issueView {
			return toIssueView(item)
		}).Values()
		return &issueOutput{Body: views}, nil
	}

	getIssue := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		item, err := service.GetIssueByIID(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	createIssue := func(ctx context.Context, in *createIssueInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateIssueInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		authorUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.AuthorUserID)
		if err != nil {
			return nil, err
		}
		input.AuthorUserID = authorUserID
		item, err := service.CreateIssue(ctx, in.ProjectID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	updateIssue := func(ctx context.Context, in *updateIssueInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.UpdateIssueInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		if input.State == nil && in.Body.Status != nil {
			mapped := statusToState(*in.Body.Status)
			input.State = &mapped
		}
		item, err := service.UpdateIssue(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	listComments := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListComments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item issuedomain.ProjectIssueComment) issueCommentView {
			return toIssueCommentView(in.IssueIID, item)
		}).Values()
		return &issueOutput{Body: views}, nil
	}

	createComment := func(ctx context.Context, in *createCommentInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateCommentInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		authorUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.AuthorUserID)
		if err != nil {
			return nil, err
		}
		input.AuthorUserID = authorUserID
		if input.Body == "" {
			input.Body = in.Body.Content
		}
		item, err := service.CreateComment(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueCommentView(in.IssueIID, item)}, nil
	}

	listAttachments := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListAttachments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: items}, nil
	}

	createAttachment := func(ctx context.Context, in *createAttachmentInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateAttachmentInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		uploadedByUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.UploadedByUserID)
		if err != nil {
			return nil, err
		}
		input.UploadedByUserID = uploadedByUserID
		item, err := service.CreateAttachment(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	}

	getAttachment := func(ctx context.Context, in *projectAttachmentInput) (*issueOutput, error) {
		item, err := service.GetAttachmentContent(ctx, in.ProjectID, in.IssueIID, in.AttachmentID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/issues", listIssues),
		httpapi.Get("/repos/{id}/issues", listIssues, httpapi.DeprecatedRoute[projectIssuesInput, issueOutput]("Use GET /projects/{id}/issues instead.")),
		httpapi.Get("/projects/{id}/issues/{issue_iid}", getIssue),
		httpapi.Get("/repos/{id}/issues/{issue_iid}", getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Get("/repos/{id}/issues/by-number/{issue_iid}", getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Post("/projects/{id}/issues", createIssue, httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues", createIssue,
			httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createIssueInput, issueOutput]("Use POST /projects/{id}/issues instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}", updateIssue, httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}", updateIssue,
			httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[updateIssueInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/comments", listComments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/comments", listComments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/comments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/comments", createComment, httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/comments", createComment,
			httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createCommentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/comments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments", listAttachments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments", listAttachments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/attachments", createAttachment, httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/attachments", createAttachment,
			httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createAttachmentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/attachments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}", getAttachment),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments/{attachment_id}", getAttachment, httpapi.DeprecatedRoute[projectAttachmentInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments/{attachment_id} instead.")),
	)
}
