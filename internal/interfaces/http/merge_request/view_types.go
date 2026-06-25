package mergerequest

type mergeRequestView struct {
	ID           string `json:"id"`
	ProjectID    string `json:"project_id"`
	IID          int64  `json:"iid"`
	AuthorUserID string `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description,omitempty"`
	State        string `json:"state"`
	SourceBranch string `json:"source_branch"`
	TargetBranch string `json:"target_branch"`
	CreatedAt    string `json:"created_at,omitempty"`
	UpdatedAt    string `json:"updated_at,omitempty"`
}

type mergeRequestDiffView struct {
	MergeRequest mergeRequestView `json:"merge_request"`
	Diff         string           `json:"diff"`
}

type mergeRequestCheckStatusView struct {
	MergeRequest           mergeRequestView                    `json:"merge_request"`
	SourceBranch           string                              `json:"source_branch"`
	SourceCommitSHA        string                              `json:"source_commit_sha"`
	TargetBranch           string                              `json:"target_branch"`
	TargetBranchProtected  bool                                `json:"target_branch_protected"`
	RequireMergeRequest    bool                                `json:"require_merge_request"`
	RequirePipelineSuccess bool                                `json:"require_pipeline_success"`
	RequireApproval        bool                                `json:"require_approval"`
	RequiredApprovals      int                                 `json:"required_approvals"`
	ApprovalCount          int                                 `json:"approval_count"`
	ApprovalRules          []mergeRequestApprovalRuleCheckView `json:"approval_rules"`
	MergeAccessLevel       string                              `json:"merge_access_level,omitempty"`
	PipelineRequired       bool                                `json:"pipeline_required"`
	Required               bool                                `json:"required"`
	Mergeable              bool                                `json:"mergeable"`
	Status                 string                              `json:"status"`
	BlockingReason         string                              `json:"blocking_reason,omitempty"`
	Blockers               []mergeRequestCheckBlockerView      `json:"blockers"`
	Pipeline               *mergeRequestPipelineView           `json:"pipeline,omitempty"`
}

type mergeRequestCheckBlockerView struct {
	Code     string `json:"code"`
	Category string `json:"category"`
	Message  string `json:"message"`
}

type mergeRequestPipelineView struct {
	ID            string `json:"id"`
	ProjectID     string `json:"project_id"`
	IID           int64  `json:"iid"`
	Name          string `json:"name"`
	Source        string `json:"source"`
	RefName       string `json:"ref_name"`
	CommitSHA     string `json:"commit_sha"`
	Status        string `json:"status"`
	ConfigSource  string `json:"config_source"`
	ConfigContent string `json:"config_content,omitempty"`
	CreatedAt     string `json:"created_at,omitempty"`
	UpdatedAt     string `json:"updated_at,omitempty"`
	StartedAt     string `json:"started_at,omitempty"`
	FinishedAt    string `json:"finished_at,omitempty"`
}

type mergeRequestParticipantView struct {
	ID             string `json:"id"`
	MergeRequestID string `json:"merge_request_id"`
	UserID         string `json:"user_id"`
	Role           string `json:"role"`
	CreatedAt      string `json:"created_at,omitempty"`
	UpdatedAt      string `json:"updated_at,omitempty"`
}

type mergeRequestParticipantsView struct {
	MergeRequest mergeRequestView              `json:"merge_request"`
	Participants []mergeRequestParticipantView `json:"participants"`
}

type mergeRequestCommentView struct {
	ID             string `json:"id"`
	MergeRequestID string `json:"merge_request_id"`
	AuthorUserID   string `json:"author_user_id"`
	Body           string `json:"body"`
	CreatedAt      string `json:"created_at,omitempty"`
	UpdatedAt      string `json:"updated_at,omitempty"`
}

type mergeRequestCommentsView struct {
	MergeRequest mergeRequestView          `json:"merge_request"`
	Comments     []mergeRequestCommentView `json:"comments"`
}

type mergeRequestApprovalView struct {
	ID             string `json:"id"`
	MergeRequestID string `json:"merge_request_id"`
	UserID         string `json:"user_id"`
	CreatedAt      string `json:"created_at,omitempty"`
	UpdatedAt      string `json:"updated_at,omitempty"`
}

type mergeRequestApprovalsView struct {
	MergeRequest mergeRequestView           `json:"merge_request"`
	Approvals    []mergeRequestApprovalView `json:"approvals"`
}

type mergeRequestApprovalRuleView struct {
	ID                string   `json:"id"`
	ProjectID         string   `json:"project_id"`
	Name              string   `json:"name"`
	TargetBranch      string   `json:"target_branch"`
	ApprovalsRequired int      `json:"approvals_required"`
	EligibleUserIDs   []string `json:"eligible_user_ids"`
	CodeOwner         bool     `json:"code_owner"`
}

type mergeRequestApprovalRulesView struct {
	ProjectID string                         `json:"project_id"`
	Rules     []mergeRequestApprovalRuleView `json:"rules"`
}

type mergeRequestApprovalRuleCheckView struct {
	RuleID            string   `json:"rule_id"`
	Name              string   `json:"name"`
	TargetBranch      string   `json:"target_branch"`
	ApprovalsRequired int      `json:"approvals_required"`
	ApprovalCount     int      `json:"approval_count"`
	EligibleUserIDs   []string `json:"eligible_user_ids"`
	CodeOwner         bool     `json:"code_owner"`
	Satisfied         bool     `json:"satisfied"`
	BlockingReason    string   `json:"blocking_reason,omitempty"`
}
