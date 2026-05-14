package mergerequest

import (
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type mergeRequestInput struct {
	ProjectID     int64  `path:"id"`
	MergeIID      int64  `path:"merge_iid"`
	Authorization string `header:"Authorization"`
}

type mergeRequestsInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type createMergeRequestInput struct {
	ProjectID     int64                  `path:"id"`
	Authorization string                 `header:"Authorization"`
	Body          createMergeRequestBody `json:"body"`
}

type updateMergeRequestInput struct {
	ProjectID     int64                  `path:"id"`
	MergeIID      int64                  `path:"merge_iid"`
	Authorization string                 `header:"Authorization"`
	Body          updateMergeRequestBody `json:"body"`
}

type mergeMergeRequestInput struct {
	ProjectID     int64                 `path:"id"`
	MergeIID      int64                 `path:"merge_iid"`
	Authorization string                `header:"Authorization"`
	Body          mergeMergeRequestBody `json:"body"`
}

type createMergeRequestCommentInput struct {
	ProjectID     int64                         `path:"id"`
	MergeIID      int64                         `path:"merge_iid"`
	Authorization string                        `header:"Authorization"`
	Body          createMergeRequestCommentBody `json:"body"`
}

type mergeRequestApprovalInput struct {
	ProjectID     int64                    `path:"id"`
	MergeIID      int64                    `path:"merge_iid"`
	Authorization string                   `header:"Authorization"`
	Body          mergeRequestApprovalBody `json:"body"`
}

type approvalRulesInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type approvalRuleInput struct {
	ProjectID     int64  `path:"id"`
	RuleID        int64  `path:"rule_id"`
	Authorization string `header:"Authorization"`
}

type createApprovalRuleInput struct {
	ProjectID     int64            `path:"id"`
	Authorization string           `header:"Authorization"`
	Body          approvalRuleBody `json:"body"`
}

type updateApprovalRuleInput struct {
	ProjectID     int64                  `path:"id"`
	RuleID        int64                  `path:"rule_id"`
	Authorization string                 `header:"Authorization"`
	Body          updateApprovalRuleBody `json:"body"`
}

type setParticipantsInput struct {
	ProjectID     int64            `path:"id"`
	MergeIID      int64            `path:"merge_iid"`
	Authorization string           `header:"Authorization"`
	Body          participantsBody `json:"body"`
}

type mergeRequestOutput struct {
	Body any `json:"body"`
}

type createMergeRequestBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
	SourceBranch string `json:"source_branch"`
	TargetBranch string `json:"target_branch"`
}

type updateMergeRequestBody struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
}

type mergeMergeRequestBody struct {
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
}

type createMergeRequestCommentBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Body         string `json:"body"`
	Content      string `json:"content"`
}

type mergeRequestApprovalBody struct {
	UserID int64 `json:"user_id"`
}

type approvalRuleBody struct {
	Name              string  `json:"name"`
	TargetBranch      string  `json:"target_branch"`
	ApprovalsRequired int     `json:"approvals_required"`
	EligibleUserIDs   []int64 `json:"eligible_user_ids"`
	CodeOwner         bool    `json:"code_owner"`
}

type updateApprovalRuleBody struct {
	Name              *string  `json:"name"`
	TargetBranch      *string  `json:"target_branch"`
	ApprovalsRequired *int     `json:"approvals_required"`
	EligibleUserIDs   *[]int64 `json:"eligible_user_ids"`
	CodeOwner         *bool    `json:"code_owner"`
}

type participantsBody struct {
	UserIDs []int64 `json:"user_ids"`
}

type Endpoint struct {
	service        *mergerequestservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *mergerequestservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Merge Requests", "Merge Requests", "Project merge request APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)

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
		httpapi.Get("/projects/{id}/merge-request-approval-rules", e.listApprovalRules, httpapi.RequireProjectActionRoute[approvalRulesInput, mergeRequestOutput]("require_merge_request_approval_rule_read", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Post("/projects/{id}/merge-request-approval-rules", e.createApprovalRule, httpapi.RequireProjectActionRoute[createApprovalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Patch("/projects/{id}/merge-request-approval-rules/{rule_id}", e.updateApprovalRule, httpapi.RequireProjectActionRoute[updateApprovalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
		httpapi.Delete("/projects/{id}/merge-request-approval-rules/{rule_id}", e.deleteApprovalRule, httpapi.RequireProjectActionRoute[approvalRuleInput, mergeRequestOutput]("require_merge_request_approval_rule_admin", authRuntime, projectScope, infraauth.ProjectActionMergeRequestMerge)),
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

func (in mergeRequestsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestsInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeMergeRequestInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeMergeRequestInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createMergeRequestCommentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createMergeRequestCommentInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in mergeRequestApprovalInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in mergeRequestApprovalInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in approvalRulesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in approvalRulesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in approvalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in approvalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createApprovalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createApprovalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateApprovalRuleInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateApprovalRuleInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in setParticipantsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in setParticipantsInput) ProjectIDValue() int64 {
	return in.ProjectID
}
