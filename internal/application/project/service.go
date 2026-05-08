package project

import (
	"context"
	"log/slog"
	"strings"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
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
