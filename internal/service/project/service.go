package project

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	"github.com/DaiYuANg/gity/internal/entity"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/repository/projectbranchprotection"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
)

var projectVisibilities = setx.NewSet("private", "internal", "public")

type Service struct {
	logger        *slog.Logger
	repo          *projectrepo.Repository
	gitRunner     *gitexec.Runner
	gitRepository *gitrepo.Service
	namespaceRepo *namespacerepo.Repository
	branchRepo    *projectbranchprotectionrepo.Repository
}

type CreateInput struct {
	NamespaceID   int64  `json:"namespace_id"`
	Name          string `json:"name"`
	PathKey       string `json:"path_key"`
	Visibility    string `json:"visibility"`
	Description   string `json:"description"`
	DefaultBranch string `json:"default_branch"`
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

func NewService(logger *slog.Logger, repo *projectrepo.Repository, gitRunner *gitexec.Runner, gitRepository *gitrepo.Service, namespaceRepo *namespacerepo.Repository, branchRepo *projectbranchprotectionrepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, gitRunner: gitRunner, gitRepository: gitRepository, namespaceRepo: namespaceRepo, branchRepo: branchRepo}
}

func (s *Service) List(ctx context.Context, namespaceID *int64) (*collectionx.List[entity.Project], error) {
	filter := sql.NullInt64{}
	if namespaceID != nil {
		filter.Valid = true
		filter.Int64 = *namespaceID
	}
	return s.repo.List(ctx, filter)
}

func (s *Service) GetByID(ctx context.Context, id int64) (entity.Project, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (entity.Project, error) {
	if input.NamespaceID <= 0 {
		return entity.Project{}, fmt.Errorf("project namespace_id is required")
	}
	if strings.TrimSpace(input.Name) == "" {
		return entity.Project{}, fmt.Errorf("project name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return entity.Project{}, fmt.Errorf("project path_key is required")
	}
	visibility := strings.TrimSpace(strings.ToLower(input.Visibility))
	if visibility == "" {
		visibility = "private"
	}
	if !projectVisibilities.Contains(visibility) {
		return entity.Project{}, fmt.Errorf("unsupported project visibility: %s", visibility)
	}
	namespace, err := s.namespaceRepo.GetByID(ctx, input.NamespaceID)
	if err != nil {
		return entity.Project{}, fmt.Errorf("load project namespace: %w", err)
	}
	project, err := s.repo.Create(ctx, projectrepo.CreateInput{
		NamespaceID:   input.NamespaceID,
		Name:          input.Name,
		PathKey:       input.PathKey,
		Visibility:    visibility,
		Description:   input.Description,
		DefaultBranch: input.DefaultBranch,
	}, namespace)
	if err != nil {
		return entity.Project{}, err
	}
	repoPath := project.FullPath + ".git"
	if err := s.gitRunner.InitBare(ctx, repoPath, project.DefaultBranch); err != nil {
		_ = s.repo.DeleteByID(ctx, project.ID)
		return entity.Project{}, fmt.Errorf("provision bare repo: %w", err)
	}
	return project, nil
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return fmt.Errorf("project id is required")
	}
	return s.repo.DeleteByID(ctx, id)
}

func (s *Service) ListBranches(ctx context.Context, id int64) ([]Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	gitBranches, err := s.gitRepository.ListBranches(ctx, repositoryPath(project), project.DefaultBranch)
	if err != nil {
		return nil, mapGitError(err)
	}
	protected, err := s.protectedBranchSet(ctx, id)
	if err != nil {
		return nil, err
	}
	branches := make([]Branch, 0, len(gitBranches))
	for _, branch := range gitBranches {
		branches = append(branches, Branch{
			Name:          branch.Name,
			Hash:          branch.Hash,
			IsDefault:     branch.IsDefault,
			IsProtected:   protected[branch.Name],
			LastCommitSHA: branch.Hash,
		})
	}
	return branches, nil
}

func (s *Service) CreateBranch(ctx context.Context, id int64, branchName string, sourceRef string) (Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return Branch{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, fmt.Errorf("branch name is required")
	}
	if strings.TrimSpace(sourceRef) == "" {
		sourceRef = project.DefaultBranch
	}
	if err := s.gitRunner.CreateBranch(ctx, repositoryPath(project), branchName, sourceRef); err != nil {
		return Branch{}, mapGitExecError(err)
	}
	return s.GetBranch(ctx, id, branchName)
}

func (s *Service) SetBranchProtection(ctx context.Context, id int64, branchName string, protected bool) (Branch, error) {
	if _, err := s.GetBranch(ctx, id, branchName); err != nil {
		return Branch{}, err
	}
	if protected {
		if _, err := s.branchRepo.Protect(ctx, id, branchName); err != nil {
			return Branch{}, err
		}
	} else if err := s.branchRepo.Unprotect(ctx, id, branchName); err != nil {
		return Branch{}, err
	}
	return s.GetBranch(ctx, id, branchName)
}

func (s *Service) GetBranch(ctx context.Context, id int64, branchName string) (Branch, error) {
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, fmt.Errorf("branch name is required")
	}
	branches, err := s.ListBranches(ctx, id)
	if err != nil {
		return Branch{}, err
	}
	for _, branch := range branches {
		if branch.Name == branchName {
			return branch, nil
		}
	}
	return Branch{}, httpx.NewError(http.StatusNotFound, "branch not found", gitrepo.ErrReferenceNotFound)
}

func (s *Service) CreateFileCommit(ctx context.Context, id int64, input CreateFileCommitInput) error {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	branchName := strings.TrimSpace(input.BranchName)
	if branchName == "" {
		branchName = project.DefaultBranch
	}
	protected, err := s.isBranchProtected(ctx, id, branchName)
	if err != nil {
		return err
	}
	if protected {
		return httpx.NewError(http.StatusForbidden, "protected branch cannot be updated", fmt.Errorf("branch is protected: %s", branchName))
	}
	err = s.gitRunner.CreateFileCommit(ctx, repositoryPath(project), gitexec.CreateFileCommitInput{
		BranchName:  branchName,
		FilePath:    input.Path,
		Content:     input.Content,
		Message:     input.Message,
		AuthorName:  input.AuthorName,
		AuthorEmail: input.AuthorEmail,
	})
	if err != nil {
		return mapGitExecError(err)
	}
	return nil
}

func (s *Service) isBranchProtected(ctx context.Context, projectID int64, branchName string) (bool, error) {
	if _, err := s.branchRepo.GetByProjectAndBranch(ctx, projectID, branchName); err == nil {
		return true, nil
	} else if err != dbxrepo.ErrNotFound {
		return false, err
	}
	return false, nil
}

func (s *Service) ListTree(ctx context.Context, id int64, refName string, treePath string) ([]gitrepo.TreeEntry, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	entries, err := s.gitRepository.ListTree(ctx, repositoryPath(project), refName, project.DefaultBranch, treePath)
	if err != nil {
		return nil, mapGitError(err)
	}
	return entries, nil
}

func (s *Service) Search(ctx context.Context, id int64, refName string, query string, path string, limit int, maxFiles int, maxFileSize int64, matchCase bool, useRegex bool) ([]gitrepo.SearchResult, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	results, err := s.gitRepository.Search(ctx, repositoryPath(project), refName, project.DefaultBranch, gitrepo.SearchParams{
		Query:       query,
		Path:        path,
		Limit:       limit,
		MaxFiles:    maxFiles,
		MaxFileSize: maxFileSize,
		MatchCase:   matchCase,
		UseRegex:    useRegex,
	})
	if err != nil {
		return nil, err
	}
	return results, nil
}

func (s *Service) GetBlob(ctx context.Context, id int64, refName string, blobPath string) (gitrepo.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitrepo.Blob{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	blob, err := s.gitRepository.GetBlob(ctx, repositoryPath(project), refName, project.DefaultBranch, blobPath)
	if err != nil {
		return gitrepo.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) GetReadme(ctx context.Context, id int64, refName string) (gitrepo.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitrepo.Blob{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	blob, err := s.gitRepository.GetReadme(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitrepo.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) ListCommits(ctx context.Context, id int64, refName string, limit int) ([]gitrepo.Commit, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	commits, err := s.gitRepository.ListCommits(ctx, repositoryPath(project), refName, project.DefaultBranch, limit)
	if err != nil {
		return nil, mapGitError(err)
	}
	return commits, nil
}

func (s *Service) AnalyzeLanguages(ctx context.Context, id int64, refName string) (gitrepo.LanguageAnalysis, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitrepo.LanguageAnalysis{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	analysis, err := s.gitRepository.AnalyzeLanguages(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitrepo.LanguageAnalysis{}, mapGitError(err)
	}
	return analysis, nil
}

func repositoryPath(project entity.Project) string {
	return project.FullPath + ".git"
}

func mapGitError(err error) error {
	switch {
	case errors.Is(err, gitrepo.ErrRepositoryNotFound):
		return httpx.NewError(http.StatusNotFound, "repository not found", err)
	case errors.Is(err, gitrepo.ErrReferenceNotFound):
		return httpx.NewError(http.StatusNotFound, "git reference not found", err)
	case errors.Is(err, gitrepo.ErrEmptyRepository):
		return httpx.NewError(http.StatusNotFound, "repository has no commits", err)
	case errors.Is(err, gitrepo.ErrInvalidSearchQuery):
		return httpx.NewError(http.StatusBadRequest, "invalid search query", err)
	case errors.Is(err, gitrepo.ErrInvalidSearchRegexp):
		return httpx.NewError(http.StatusBadRequest, "invalid search regex", err)
	case errors.Is(err, gitrepo.ErrPathNotFound):
		return httpx.NewError(http.StatusNotFound, "repository path not found", err)
	case errors.Is(err, gitrepo.ErrReadmeNotFound):
		return httpx.NewError(http.StatusNotFound, "repository readme not found", err)
	default:
		return err
	}
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitexec.ErrBranchExists):
		return httpx.NewError(http.StatusConflict, "branch already exists", err)
	case errors.Is(err, gitexec.ErrInvalidBranchName):
		return httpx.NewError(http.StatusBadRequest, "invalid branch name", err)
	case errors.Is(err, gitexec.ErrSourceReferenceNotFound):
		return httpx.NewError(http.StatusNotFound, "git reference not found", err)
	case errors.Is(err, gitexec.ErrFileAlreadyExists):
		return httpx.NewError(http.StatusConflict, "repository file already exists", err)
	default:
		return err
	}
}

func (s *Service) protectedBranchSet(ctx context.Context, projectID int64) (map[string]bool, error) {
	items, err := s.branchRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	protected := map[string]bool{}
	items.Range(func(_ int, item entity.ProjectBranchProtection) bool {
		protected[item.BranchName] = true
		return true
	})
	return protected, nil
}
