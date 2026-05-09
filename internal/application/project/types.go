package project

import "log/slog"

import gitports "github.com/DaiYuANg/gity/internal/application/ports"

type GitDependencies struct {
	Runner     gitports.GitRunner
	Repository gitports.GitRepository
}

type Dependencies struct {
	Logger           *slog.Logger
	Repo             gitports.ProjectRepository
	Git              GitDependencies
	OrganizationRepo gitports.OrganizationRepository
	BranchRepo       gitports.ProjectBranchProtectionRepository
	Runtime          RuntimeDependencies
}

type RuntimeDependencies struct {
	Events      gitports.DomainEventPublisher
	SearchIndex gitports.CodeSearchIndex
}

type CreateInput struct {
	OrganizationID int64  `json:"organization_id"`
	Name           string `json:"name"`
	PathKey        string `json:"path_key"`
	Visibility     string `json:"visibility"`
	Description    string `json:"description"`
	DefaultBranch  string `json:"default_branch"`
}

type DeleteInput struct {
	Confirmation string `json:"confirmation"`
}

type BranchProtection struct {
	ID                     int64  `json:"id"`
	ProjectID              int64  `json:"project_id"`
	BranchName             string `json:"branch_name"`
	RuleType               string `json:"rule_type"`
	PushAccessLevel        string `json:"push_access_level"`
	MergeAccessLevel       string `json:"merge_access_level"`
	RequireMergeRequest    bool   `json:"require_merge_request"`
	RequirePipelineSuccess bool   `json:"require_pipeline_success"`
	AllowForcePush         bool   `json:"allow_force_push"`
	AllowDelete            bool   `json:"allow_delete"`
}

type BranchProtectionInput struct {
	BranchName             string `json:"branch_name"`
	RuleType               string `json:"rule_type"`
	PushAccessLevel        string `json:"push_access_level"`
	MergeAccessLevel       string `json:"merge_access_level"`
	RequireMergeRequest    bool   `json:"require_merge_request"`
	RequirePipelineSuccess bool   `json:"require_pipeline_success"`
	AllowForcePush         bool   `json:"allow_force_push"`
	AllowDelete            bool   `json:"allow_delete"`
}

type Branch struct {
	Name          string            `json:"name"`
	Hash          string            `json:"hash"`
	IsDefault     bool              `json:"is_default"`
	IsProtected   bool              `json:"is_protected"`
	LastCommitSHA string            `json:"last_commit_sha"`
	Protection    *BranchProtection `json:"protection,omitempty"`
}

type CreateFileCommitInput struct {
	BranchName  string `json:"branch_name"`
	Path        string `json:"path"`
	Content     string `json:"content"`
	Message     string `json:"message"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
}
