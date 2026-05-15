package mergerequest

import (
	"github.com/arcgolabs/httpx"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)
	e.registerReadRoutes(registrar, authRuntime, projectScope)
	e.registerApprovalRuleRoutes(registrar, authRuntime, projectScope)
	e.registerWriteRoutes(registrar, authRuntime, projectScope)
}

func (e *Endpoint) registerReadRoutes(registrar httpx.Registrar, authRuntime *infraauth.Runtime, projectScope httpapi.ProjectScopeResolver) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/merge-requests", e.listMergeRequests, httpapi.RequireProjectReadRoute[mergeRequestsInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests", e.listMergeRequests,
			httpapi.RequireProjectReadRoute[mergeRequestsInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestsInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}", e.getMergeRequest, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}", e.getMergeRequest,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid} instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/diff", e.getDiff, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/diff", e.getDiff,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/diff instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/checks", e.getChecks, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/checks", e.getChecks,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/checks instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/participants", e.listParticipants, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/participants", e.listParticipants,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/participants instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/comments", e.listComments, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/comments", e.listComments,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/comments instead."),
		),
		httpapi.Get("/projects/{id}/merge-requests/{merge_iid}/approvals", e.listApprovals, httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope)),
		httpapi.Get("/repos/{id}/merge-requests/{merge_iid}/approvals", e.listApprovals,
			httpapi.RequireProjectReadRoute[mergeRequestInput, mergeRequestOutput](authRuntime, projectScope),
			httpapi.DeprecatedRoute[mergeRequestInput, mergeRequestOutput]("Use GET /projects/{id}/merge-requests/{merge_iid}/approvals instead."),
		),
	)
}

func (e *Endpoint) registerApprovalRuleRoutes(registrar httpx.Registrar, authRuntime *infraauth.Runtime, projectScope httpapi.ProjectScopeResolver) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/merge-request-approval-rules", e.listApprovalRules, httpapi.RequireProjectActionRoute[approvalRulesInput, mergeRequestOutput]("require_merge_request_approval_rule_read", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Post("/projects/{id}/merge-request-approval-rules", e.createApprovalRule, httpapi.RequireProjectActionRoute[createApprovalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Patch("/projects/{id}/merge-request-approval-rules/{rule_id}", e.updateApprovalRule, httpapi.RequireProjectActionRoute[updateApprovalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Delete("/projects/{id}/merge-request-approval-rules/{rule_id}", e.deleteApprovalRule, httpapi.RequireProjectActionRoute[approvalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
	)
}

func (e *Endpoint) registerWriteRoutes(registrar httpx.Registrar, authRuntime *infraauth.Runtime, projectScope httpapi.ProjectScopeResolver) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Post("/projects/{id}/merge-requests", e.createMergeRequest, httpapi.RequireProjectActionRoute[createMergeRequestInput, mergeRequestOutput]("require_merge_request_create", authRuntime, projectScope, infraauth.ProjectActionMergeRequestCreate)),
		httpapi.Post("/repos/{id}/merge-requests", e.createMergeRequest,
			httpapi.RequireProjectActionRoute[createMergeRequestInput, mergeRequestOutput]("require_merge_request_create", authRuntime, projectScope, infraauth.ProjectActionMergeRequestCreate),
			httpapi.DeprecatedRoute[createMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/comments", e.createComment, httpapi.RequireProjectActionRoute[createMergeRequestCommentInput, mergeRequestOutput]("require_merge_request_comment", authRuntime, projectScope, infraauth.ProjectActionMergeRequestComment)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/comments", e.createComment,
			httpapi.RequireProjectActionRoute[createMergeRequestCommentInput, mergeRequestOutput]("require_merge_request_comment", authRuntime, projectScope, infraauth.ProjectActionMergeRequestComment),
			httpapi.DeprecatedRoute[createMergeRequestCommentInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/comments instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/approve", e.approve, httpapi.RequireProjectActionRoute[mergeRequestApprovalInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/approve", e.approve,
			httpapi.RequireProjectActionRoute[mergeRequestApprovalInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite),
			httpapi.DeprecatedRoute[mergeRequestApprovalInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/approve instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/unapprove", e.unapprove, httpapi.RequireProjectActionRoute[mergeRequestApprovalInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/unapprove", e.unapprove,
			httpapi.RequireProjectActionRoute[mergeRequestApprovalInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite),
			httpapi.DeprecatedRoute[mergeRequestApprovalInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/unapprove instead."),
		),
		httpapi.Post("/projects/{id}/merge-requests/{merge_iid}/merge", e.mergeMergeRequest, httpapi.RequireProjectActionRoute[mergeMergeRequestInput, mergeRequestOutput]("require_merge_request_merge", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Post("/repos/{id}/merge-requests/{merge_iid}/merge", e.mergeMergeRequest,
			httpapi.RequireProjectActionRoute[mergeMergeRequestInput, mergeRequestOutput]("require_merge_request_merge", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge),
			httpapi.DeprecatedRoute[mergeMergeRequestInput, mergeRequestOutput]("Use POST /projects/{id}/merge-requests/{merge_iid}/merge instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}/reviewers", e.setReviewers, httpapi.RequireProjectActionRoute[setParticipantsInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}/reviewers", e.setReviewers,
			httpapi.RequireProjectActionRoute[setParticipantsInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite),
			httpapi.DeprecatedRoute[setParticipantsInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid}/reviewers instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}/assignees", e.setAssignees, httpapi.RequireProjectActionRoute[setParticipantsInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}/assignees", e.setAssignees,
			httpapi.RequireProjectActionRoute[setParticipantsInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite),
			httpapi.DeprecatedRoute[setParticipantsInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid}/assignees instead."),
		),
		httpapi.Patch("/projects/{id}/merge-requests/{merge_iid}", e.updateMergeRequest, httpapi.RequireProjectActionRoute[updateMergeRequestInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite)),
		httpapi.Patch("/repos/{id}/merge-requests/{merge_iid}", e.updateMergeRequest,
			httpapi.RequireProjectActionRoute[updateMergeRequestInput, mergeRequestOutput]("require_merge_request_write", authRuntime, projectScope, infraauth.ProjectActionMergeRequestWrite),
			httpapi.DeprecatedRoute[updateMergeRequestInput, mergeRequestOutput]("Use PATCH /projects/{id}/merge-requests/{merge_iid} instead."),
		),
	)
}
