package ports

import (
	"context"
	"errors"
	"io"
)

var (
	ErrRepositoryNotFound      = errors.New("git repository not found")
	ErrReferenceNotFound       = errors.New("git reference not found")
	ErrPathNotFound            = errors.New("git path not found")
	ErrReadmeNotFound          = errors.New("git readme not found")
	ErrEmptyRepository         = errors.New("git repository is empty")
	ErrInvalidSearchQuery      = errors.New("invalid search query")
	ErrInvalidSearchRegexp     = errors.New("invalid search regex")
	ErrBranchExists            = errors.New("git branch already exists")
	ErrInvalidBranchName       = errors.New("invalid git branch name")
	ErrSourceReferenceNotFound = errors.New("git source reference not found")
	ErrFileAlreadyExists       = errors.New("git file already exists")
	ErrMergeConflict           = errors.New("git merge conflict")
)

type GitRepository interface {
	ListBranches(ctx context.Context, repoPath string, defaultBranch string) ([]Branch, error)
	ListTree(ctx context.Context, repoPath string, refName string, defaultBranch string, treePath string) ([]TreeEntry, error)
	GetBlob(ctx context.Context, repoPath string, refName string, defaultBranch string, blobPath string) (Blob, error)
	GetReadme(ctx context.Context, repoPath string, refName string, defaultBranch string) (Blob, error)
	ListCommits(ctx context.Context, repoPath string, refName string, defaultBranch string, limit int) ([]Commit, error)
	Search(ctx context.Context, repoPath string, refName string, defaultBranch string, input SearchParams) ([]SearchResult, error)
	AnalyzeLanguages(ctx context.Context, repoPath string, refName string, defaultBranch string) (LanguageAnalysis, error)
}

type GitRunner interface {
	Run(ctx context.Context, repoPath string, args []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error
	InitBare(ctx context.Context, repoPath string, initialBranch string) error
	CreateBranch(ctx context.Context, repoPath string, branchName string, sourceRef string) error
	CreateFileCommit(ctx context.Context, repoPath string, input CreateFileCommitInput) error
	DiffBranches(ctx context.Context, repoPath string, targetBranch string, sourceBranch string) (string, error)
	Archive(ctx context.Context, repoPath string, revision string) ([]byte, error)
	MergeBranches(ctx context.Context, repoPath string, input MergeBranchesInput) error
}

type Branch struct {
	Name      string `json:"name"`
	Hash      string `json:"hash"`
	IsDefault bool   `json:"is_default"`
}

type TreeEntry struct {
	Name string `json:"name"`
	Path string `json:"path"`
	Type string `json:"type"`
	Mode string `json:"mode"`
	Size int64  `json:"size"`
}

type Blob struct {
	Name     string `json:"name"`
	Path     string `json:"path"`
	Encoding string `json:"encoding"`
	Size     int64  `json:"size"`
	Content  string `json:"content"`
}

type Commit struct {
	Hash        string `json:"hash"`
	ShortHash   string `json:"short_hash"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
	CommittedAt string `json:"committed_at"`
}

type SearchResult struct {
	Path        string `json:"path"`
	LineNumber  int    `json:"line_number"`
	Column      int    `json:"column"`
	MatchLength int    `json:"match_length"`
	LineContent string `json:"line_content"`
}

type LanguageStat struct {
	Language   string  `json:"language"`
	Bytes      int64   `json:"bytes"`
	Percentage float64 `json:"percentage"`
}

type LanguageAnalysis struct {
	Revision   string         `json:"revision"`
	TotalBytes int64          `json:"total_bytes"`
	Languages  []LanguageStat `json:"languages"`
}

type SearchParams struct {
	Query       string `json:"query"`
	Path        string `json:"path"`
	Limit       int    `json:"limit"`
	MaxFiles    int    `json:"max_files"`
	MaxFileSize int64  `json:"max_file_size"`
	MatchCase   bool   `json:"match_case"`
	UseRegex    bool   `json:"regex"`
}

type CreateFileCommitInput struct {
	BranchName  string
	FilePath    string
	Content     string
	Message     string
	AuthorName  string
	AuthorEmail string
}

type MergeBranchesInput struct {
	TargetBranch string
	SourceBranch string
	Message      string
	AuthorName   string
	AuthorEmail  string
}
