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
