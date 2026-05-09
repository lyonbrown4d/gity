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
	Events           gitports.DomainEventPublisher
}

type CreateInput struct {
	OrganizationID int64  `json:"organization_id"`
	Name           string `json:"name"`
	PathKey        string `json:"path_key"`
	Visibility     string `json:"visibility"`
	Description    string `json:"description"`
	DefaultBranch  string `json:"default_branch"`
}

type Branch struct {
	Name          string `json:"name"`
	Hash          string `json:"hash"`
	IsDefault     bool   `json:"is_default"`
	IsProtected   bool   `json:"is_protected"`
	LastCommitSHA string `json:"last_commit_sha"`
}

type CreateFileCommitInput struct {
	BranchName  string `json:"branch_name"`
	Path        string `json:"path"`
	Content     string `json:"content"`
	Message     string `json:"message"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
}
