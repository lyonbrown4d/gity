package project

import "github.com/DaiYuANg/gity/internal/infrastructure/git_repo"

type createProjectInput struct {
	Authorization string            `header:"Authorization"`
	Body          createProjectBody `json:"body"`
}

type projectByIDInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type projectsInput struct {
	OrganizationID int64  `query:"organization_id"`
	NamespaceID    int64  `query:"namespace_id"`
	IDs            string `query:"ids"`
}

type projectRepositoryInput struct {
	ID     int64  `path:"id"`
	Ref    string `query:"ref"`
	Branch string `query:"branch_name"`
	Path   string `query:"path"`
	Limit  int    `query:"limit"`
}

type createBranchInput struct {
	ID            int64            `path:"id"`
	Authorization string           `header:"Authorization"`
	Body          createBranchBody `json:"body"`
}

type branchProtectionInput struct {
	ID            int64  `path:"id"`
	BranchName    string `path:"branch_name"`
	Authorization string `header:"Authorization"`
}

type createFileCommitInput struct {
	ID            int64                `path:"id"`
	Authorization string               `header:"Authorization"`
	Body          createFileCommitBody `json:"body"`
}

type projectRepositorySearchInput struct {
	ID          int64  `path:"id"`
	Ref         string `query:"ref"`
	Branch      string `query:"branch_name"`
	Query       string `query:"query"`
	Path        string `query:"path"`
	Limit       int    `query:"limit"`
	MaxFiles    int    `query:"max_files"`
	MaxFileSize int64  `query:"max_file_size"`
	MatchCase   bool   `query:"match_case"`
	Regex       bool   `query:"regex"`
}

type projectOutput struct {
	Body any `json:"body"`
}

type createProjectBody struct {
	OrganizationID int64  `json:"organization_id"`
	NamespaceID    int64  `json:"namespace_id"`
	Key            string `json:"key"`
	Name           string `json:"name"`
	PathKey        string `json:"path_key"`
	Visibility     string `json:"visibility"`
	Description    string `json:"description"`
	DefaultBranch  string `json:"default_branch"`
}

type createBranchBody struct {
	Name      string `json:"name"`
	SourceRef string `json:"source_ref"`
}

type createFileCommitBody struct {
	BranchName  string `json:"branch_name"`
	Path        string `json:"path"`
	Content     string `json:"content"`
	Message     string `json:"message"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
}

type repositoryView struct {
	ID             string `json:"id"`
	UUID           string `json:"uuid"`
	OrganizationID string `json:"organization_id"`
	Key            string `json:"key"`
	Name           string `json:"name"`
	Description    string `json:"description"`
	Visibility     string `json:"visibility"`
	DefaultBranch  string `json:"default_branch"`
	CloneHTTPURL   string `json:"clone_http_url"`
}

type repositoryBranchView struct {
	RepositoryID  string `json:"repository_id"`
	Name          string `json:"name"`
	IsProtected   bool   `json:"is_protected"`
	LastCommitSHA string `json:"last_commit_sha,omitempty"`
}

type repositoryCommitView struct {
	RepositoryID string `json:"repository_id"`
	BranchName   string `json:"branch_name"`
	CommitSHA    string `json:"commit_sha"`
	Message      string `json:"message"`
	AuthorUserID string `json:"author_user_id"`
	CreatedAt    string `json:"created_at"`
}

type repositoryTreeEntryView struct {
	Name string `json:"name"`
	Path string `json:"path"`
	Kind string `json:"kind"`
	OID  string `json:"oid"`
	Size int64  `json:"size,omitempty"`
}

type repositoryBlobView struct {
	Path     string `json:"path"`
	Content  string `json:"content"`
	Size     int64  `json:"size"`
	IsBinary bool   `json:"is_binary"`
	Encoding string `json:"encoding"`
}

type repositoryLanguagesView struct {
	RepositoryID string                 `json:"repository_id"`
	BranchName   string                 `json:"branch_name"`
	Status       string                 `json:"status"`
	Revision     string                 `json:"revision,omitempty"`
	AnalyzedAt   string                 `json:"analyzed_at,omitempty"`
	TotalBytes   int64                  `json:"total_bytes"`
	Languages    []gitrepo.LanguageStat `json:"languages"`
}
