package project

import (
	"context"
	"errors"
	"fmt"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
	"log/slog"
	"strings"
)

var projectVisibilities = setx.NewSet("private", "internal", "public")

type Service struct {
	logger        *slog.Logger
	repo          gitports.ProjectRepository
	gitRunner     gitports.GitRunner
	gitRepository gitports.GitRepository
	namespaceRepo gitports.NamespaceRepository
	branchRepo    gitports.ProjectBranchProtectionRepository
	events        gitports.DomainEventPublisher
}

type GitDependencies struct {
	Runner     gitports.GitRunner
	Repository gitports.GitRepository
}

type Dependencies struct {
	Logger        *slog.Logger
	Repo          gitports.ProjectRepository
	Git           GitDependencies
	NamespaceRepo gitports.NamespaceRepository
	BranchRepo    gitports.ProjectBranchProtectionRepository
	Events        gitports.DomainEventPublisher
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

func NewGitDependencies(runner gitports.GitRunner, repository gitports.GitRepository) GitDependencies {
	return GitDependencies{Runner: runner, Repository: repository}
}

func NewDependencies(logger *slog.Logger, repo gitports.ProjectRepository, git GitDependencies, namespaceRepo gitports.NamespaceRepository, branchRepo gitports.ProjectBranchProtectionRepository, events gitports.DomainEventPublisher) Dependencies {
	return Dependencies{Logger: logger, Repo: repo, Git: git, NamespaceRepo: namespaceRepo, BranchRepo: branchRepo, Events: events}
}

func NewServiceWithDependencies(deps Dependencies) *Service {
	return newService(deps)
}

func NewService(logger *slog.Logger, repo gitports.ProjectRepository, gitRunner gitports.GitRunner, gitRepository gitports.GitRepository, namespaceRepo gitports.NamespaceRepository, branchRepo gitports.ProjectBranchProtectionRepository) *Service {
	return newService(Dependencies{
		Logger:        logger,
		Repo:          repo,
		Git:           NewGitDependencies(gitRunner, gitRepository),
		NamespaceRepo: namespaceRepo,
		BranchRepo:    branchRepo,
		Events:        gitports.NoopDomainEventPublisher{},
	})
}

func newService(deps Dependencies) *Service {
	events := deps.Events
	if events == nil {
		events = gitports.NoopDomainEventPublisher{}
	}
	return &Service{
		logger:        deps.Logger,
		repo:          deps.Repo,
		gitRunner:     deps.Git.Runner,
		gitRepository: deps.Git.Repository,
		namespaceRepo: deps.NamespaceRepo,
		branchRepo:    deps.BranchRepo,
		events:        events,
	}
}

func (s *Service) List(ctx context.Context, namespaceID *int64) (*collectionx.List[projectdomain.Project], error) {
	items, err := s.repo.List(ctx, namespaceID)
	if err != nil {
		return nil, oops.In("project").With("namespace_id", namespaceID).Wrapf(err, "list projects")
	}
	return items, nil
}

func (s *Service) GetByID(ctx context.Context, id int64) (projectdomain.Project, error) {
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return projectdomain.Project{}, oops.In("project").With("project_id", id).Wrapf(err, "load project")
	}
	return item, nil
}

func (s *Service) Create(ctx context.Context, input CreateInput) (projectdomain.Project, error) {
	if input.NamespaceID <= 0 {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID).New("project namespace_id is required")
	}
	if strings.TrimSpace(input.Name) == "" {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID).New("project name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID, "name", input.Name).New("project path_key is required")
	}
	visibility := strings.TrimSpace(strings.ToLower(input.Visibility))
	if visibility == "" {
		visibility = "private"
	}
	if !projectVisibilities.Contains(visibility) {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID, "visibility", visibility).New("unsupported project visibility")
	}
	namespace, err := s.namespaceRepo.GetByID(ctx, input.NamespaceID)
	if err != nil {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID).Wrapf(err, "load project namespace")
	}
	project, err := s.repo.Create(ctx, gitports.CreateProjectInput{
		NamespaceID:   input.NamespaceID,
		Name:          input.Name,
		PathKey:       input.PathKey,
		Visibility:    visibility,
		Description:   input.Description,
		DefaultBranch: input.DefaultBranch,
	}, namespace)
	if err != nil {
		return projectdomain.Project{}, oops.In("project").With("namespace_id", input.NamespaceID, "name", input.Name, "path_key", input.PathKey).Wrapf(err, "create project")
	}
	repoPath := project.FullPath + ".git"
	if err := s.gitRunner.InitBare(ctx, repoPath, project.DefaultBranch); err != nil {
		if cleanupErr := s.repo.DeleteByID(ctx, project.ID); cleanupErr != nil {
			return projectdomain.Project{}, oops.In("project").With("project_id", project.ID, "repo_path", repoPath).Wrapf(oops.Join(err, cleanupErr), "provision bare repo and cleanup project")
		}
		return projectdomain.Project{}, oops.In("project").With("project_id", project.ID, "repo_path", repoPath).Wrapf(err, "provision bare repo")
	}
	if err := s.events.PublishAsync(ctx, projectdomain.NewProjectCreatedEvent(project)); err != nil {
		wrapped := oops.In("project").With("project_id", project.ID).Wrapf(err, "publish project created event")
		s.logger.Warn("publish project created event failed", slog.String("error", wrapped.Error()))
	}
	return project, nil
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return oops.In("project").With("project_id", id).New("project id is required")
	}
	if err := s.repo.DeleteByID(ctx, id); err != nil {
		return oops.In("project").With("project_id", id).Wrapf(err, "delete project")
	}
	if err := s.events.PublishAsync(ctx, projectdomain.NewProjectDeletedEvent(id)); err != nil {
		wrapped := oops.In("project").With("project_id", id).Wrapf(err, "publish project deleted event")
		s.logger.Warn("publish project deleted event failed", slog.String("error", wrapped.Error()))
	}
	return nil
}

func (s *Service) ListBranches(ctx context.Context, id int64) ([]Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	gitBranches, err := s.gitRepository.ListBranches(ctx, repositoryPath(project), project.DefaultBranch)
	if err != nil {
		return nil, mapGitError(err)
	}
	protected, err := s.protectedBranchSet(ctx, id)
	if err != nil {
		return nil, err
	}
	return collectionx.MapList(collectionx.NewList(gitBranches...), func(_ int, branch gitports.Branch) Branch {
		return Branch{
			Name:          branch.Name,
			Hash:          branch.Hash,
			IsDefault:     branch.IsDefault,
			IsProtected:   protected.Contains(branch.Name),
			LastCommitSHA: branch.Hash,
		}
	}).Values(), nil
}

func (s *Service) CreateBranch(ctx context.Context, id int64, branchName, sourceRef string) (Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return Branch{}, apperror.NotFound("project not found", err)
	}
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, errors.New("branch name is required")
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
			return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "protect branch")
		}
	} else if err := s.branchRepo.Unprotect(ctx, id, branchName); err != nil {
		return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "unprotect branch")
	}
	return s.GetBranch(ctx, id, branchName)
}

func (s *Service) GetBranch(ctx context.Context, id int64, branchName string) (Branch, error) {
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, errors.New("branch name is required")
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
	return Branch{}, apperror.NotFound("branch not found", gitports.ErrReferenceNotFound)
}

func (s *Service) CreateFileCommit(ctx context.Context, id int64, input CreateFileCommitInput) error {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return apperror.NotFound("project not found", err)
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
		return apperror.Forbidden("protected branch cannot be updated", fmt.Errorf("branch is protected: %s", branchName))
	}
	err = s.gitRunner.CreateFileCommit(ctx, repositoryPath(project), gitports.CreateFileCommitInput{
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
	} else if !errors.Is(err, gitports.ErrNotFound) {
		return false, oops.In("project").With("project_id", projectID, "branch", branchName).Wrapf(err, "check branch protection")
	}
	return false, nil
}

func (s *Service) ListTree(ctx context.Context, id int64, refName, treePath string) ([]gitports.TreeEntry, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	entries, err := s.gitRepository.ListTree(ctx, repositoryPath(project), refName, project.DefaultBranch, treePath)
	if err != nil {
		return nil, mapGitError(err)
	}
	return entries, nil
}

func (s *Service) Search(ctx context.Context, id int64, refName, query, path string, limit, maxFiles int, maxFileSize int64, matchCase, useRegex bool) ([]gitports.SearchResult, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	results, err := s.gitRepository.Search(ctx, repositoryPath(project), refName, project.DefaultBranch, gitports.SearchParams{
		Query:       query,
		Path:        path,
		Limit:       limit,
		MaxFiles:    maxFiles,
		MaxFileSize: maxFileSize,
		MatchCase:   matchCase,
		UseRegex:    useRegex,
	})
	if err != nil {
		return nil, mapGitError(err)
	}
	return results, nil
}

func (s *Service) GetBlob(ctx context.Context, id int64, refName, blobPath string) (gitports.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.Blob{}, apperror.NotFound("project not found", err)
	}
	blob, err := s.gitRepository.GetBlob(ctx, repositoryPath(project), refName, project.DefaultBranch, blobPath)
	if err != nil {
		return gitports.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) GetReadme(ctx context.Context, id int64, refName string) (gitports.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.Blob{}, apperror.NotFound("project not found", err)
	}
	blob, err := s.gitRepository.GetReadme(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitports.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) ListCommits(ctx context.Context, id int64, refName string, limit int) ([]gitports.Commit, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	commits, err := s.gitRepository.ListCommits(ctx, repositoryPath(project), refName, project.DefaultBranch, limit)
	if err != nil {
		return nil, mapGitError(err)
	}
	return commits, nil
}

func (s *Service) AnalyzeLanguages(ctx context.Context, id int64, refName string) (gitports.LanguageAnalysis, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.LanguageAnalysis{}, apperror.NotFound("project not found", err)
	}
	analysis, err := s.gitRepository.AnalyzeLanguages(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitports.LanguageAnalysis{}, mapGitError(err)
	}
	return analysis, nil
}

func repositoryPath(project projectdomain.Project) string {
	return project.FullPath + ".git"
}

func mapGitError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrRepositoryNotFound):
		return apperror.NotFound("repository not found", err)
	case errors.Is(err, gitports.ErrReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrEmptyRepository):
		return apperror.NotFound("repository has no commits", err)
	case errors.Is(err, gitports.ErrInvalidSearchQuery):
		return apperror.BadRequest("invalid search query", err)
	case errors.Is(err, gitports.ErrInvalidSearchRegexp):
		return apperror.BadRequest("invalid search regex", err)
	case errors.Is(err, gitports.ErrPathNotFound):
		return apperror.NotFound("repository path not found", err)
	case errors.Is(err, gitports.ErrReadmeNotFound):
		return apperror.NotFound("repository readme not found", err)
	default:
		return err
	}
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrBranchExists):
		return apperror.Conflict("branch already exists", err)
	case errors.Is(err, gitports.ErrInvalidBranchName):
		return apperror.BadRequest("invalid branch name", err)
	case errors.Is(err, gitports.ErrSourceReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrFileAlreadyExists):
		return apperror.Conflict("repository file already exists", err)
	default:
		return err
	}
}

func (s *Service) protectedBranchSet(ctx context.Context, projectID int64) (*setx.Set[string], error) {
	items, err := s.branchRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("project").With("project_id", projectID).Wrapf(err, "list protected branches")
	}
	return setx.NewSetWithCapacity[string](items.Len(), collectionx.MapList(items, func(_ int, item projectdomain.ProjectBranchProtection) string {
		return item.BranchName
	}).Values()...), nil
}
