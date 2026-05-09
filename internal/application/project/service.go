package project

import (
	"context"
	"log/slog"
	"strings"
	"time"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
)

var projectVisibilities = setx.NewSet("private", "internal", "public")

type Service struct {
	logger           *slog.Logger
	repo             gitports.ProjectRepository
	gitRunner        gitports.GitRunner
	gitRepository    gitports.GitRepository
	organizationRepo gitports.OrganizationRepository
	branchRepo       gitports.ProjectBranchProtectionRepository
	events           gitports.DomainEventPublisher
	searchIndex      gitports.CodeSearchIndex
}

func NewGitDependencies(runner gitports.GitRunner, repository gitports.GitRepository) GitDependencies {
	return GitDependencies{Runner: runner, Repository: repository}
}

func NewRuntimeDependencies(events gitports.DomainEventPublisher, searchIndex gitports.CodeSearchIndex) RuntimeDependencies {
	if events == nil {
		events = gitports.NoopDomainEventPublisher{}
	}
	return RuntimeDependencies{Events: events, SearchIndex: searchIndex}
}

func NewDependencies(logger *slog.Logger, repo gitports.ProjectRepository, git GitDependencies, organizationRepo gitports.OrganizationRepository, branchRepo gitports.ProjectBranchProtectionRepository, runtime RuntimeDependencies) Dependencies {
	return Dependencies{Logger: logger, Repo: repo, Git: git, OrganizationRepo: organizationRepo, BranchRepo: branchRepo, Runtime: runtime}
}

func NewServiceWithDependencies(deps Dependencies) *Service {
	return newService(deps)
}

func NewService(logger *slog.Logger, repo gitports.ProjectRepository, gitRunner gitports.GitRunner, gitRepository gitports.GitRepository, organizationRepo gitports.OrganizationRepository, branchRepo gitports.ProjectBranchProtectionRepository) *Service {
	return newService(Dependencies{
		Logger:           logger,
		Repo:             repo,
		Git:              NewGitDependencies(gitRunner, gitRepository),
		OrganizationRepo: organizationRepo,
		BranchRepo:       branchRepo,
		Runtime:          RuntimeDependencies{Events: gitports.NoopDomainEventPublisher{}},
	})
}

func newService(deps Dependencies) *Service {
	events := deps.Runtime.Events
	if events == nil {
		events = gitports.NoopDomainEventPublisher{}
	}
	return &Service{
		logger:           deps.Logger,
		repo:             deps.Repo,
		gitRunner:        deps.Git.Runner,
		gitRepository:    deps.Git.Repository,
		organizationRepo: deps.OrganizationRepo,
		branchRepo:       deps.BranchRepo,
		events:           events,
		searchIndex:      deps.Runtime.SearchIndex,
	}
}

func (s *Service) List(ctx context.Context, organizationID *int64) (*collectionx.List[projectdomain.Project], error) {
	items, err := s.repo.List(ctx, organizationID)
	if err != nil {
		return nil, oops.In("project").With("organization_id", organizationID).Wrapf(err, "list projects")
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
	visibility, err := validateCreateInput(input)
	if err != nil {
		return projectdomain.Project{}, err
	}
	organization, err := s.organizationRepo.GetByID(ctx, input.OrganizationID)
	if err != nil {
		return projectdomain.Project{}, oops.In("project").With("organization_id", input.OrganizationID).Wrapf(err, "load project organization")
	}
	project, err := s.repo.Create(ctx, gitports.CreateProjectInput{
		OrganizationID: input.OrganizationID,
		Name:           input.Name,
		PathKey:        input.PathKey,
		Visibility:     visibility,
		Description:    input.Description,
		DefaultBranch:  input.DefaultBranch,
	}, organization)
	if err != nil {
		return projectdomain.Project{}, oops.In("project").With("organization_id", input.OrganizationID, "name", input.Name, "path_key", input.PathKey).Wrapf(err, "create project")
	}
	if err := s.provisionRepository(ctx, project); err != nil {
		return projectdomain.Project{}, err
	}
	s.publishProjectEventAsync(ctx, project.ID, projectdomain.NewProjectCreatedEvent(project))
	return project, nil
}

func validateCreateInput(input CreateInput) (string, error) {
	if input.OrganizationID <= 0 {
		return "", oops.In("project").With("organization_id", input.OrganizationID).New("project organization_id is required")
	}
	if strings.TrimSpace(input.Name) == "" {
		return "", oops.In("project").With("organization_id", input.OrganizationID).New("project name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return "", oops.In("project").With("organization_id", input.OrganizationID, "name", input.Name).New("project path_key is required")
	}
	return normalizeVisibility(input.OrganizationID, input.Visibility)
}

func normalizeVisibility(organizationID int64, value string) (string, error) {
	visibility := strings.TrimSpace(strings.ToLower(value))
	if visibility == "" {
		visibility = "private"
	}
	if !projectVisibilities.Contains(visibility) {
		return "", oops.In("project").With("organization_id", organizationID, "visibility", visibility).New("unsupported project visibility")
	}
	return visibility, nil
}

func (s *Service) provisionRepository(ctx context.Context, project projectdomain.Project) error {
	repoPath := project.FullPath + ".git"
	if err := s.gitRunner.InitBare(ctx, repoPath, project.DefaultBranch); err != nil {
		if cleanupErr := s.repo.DeleteByID(ctx, project.ID); cleanupErr != nil {
			return oops.In("project").With("project_id", project.ID, "repo_path", repoPath).Wrapf(oops.Join(err, cleanupErr), "provision bare repo and cleanup project")
		}
		return oops.In("project").With("project_id", project.ID, "repo_path", repoPath).Wrapf(err, "provision bare repo")
	}
	return nil
}

func (s *Service) Delete(ctx context.Context, id int64, inputs ...DeleteInput) error {
	if id <= 0 {
		return oops.In("project").With("project_id", id).New("project id is required")
	}
	project, err := s.repo.GetIncludingDeletedByID(ctx, id)
	if err != nil {
		return oops.In("project").With("project_id", id).Wrapf(err, "load project for delete")
	}
	input := normalizeDeleteInput(inputs)
	if err := validateDeleteConfirmation(project, input); err != nil {
		return err
	}
	if project.IsPendingDelete() {
		return nil
	}
	deletedAt := time.Now().UTC()
	if err := s.repo.MarkPendingDeleteByID(ctx, id, deletedAt); err != nil {
		return oops.In("project").With("project_id", id, "full_path", project.FullPath).Wrapf(err, "mark project pending delete")
	}
	project.Status = projectdomain.ProjectStatusPendingDelete
	project.DeletedAt = deletedAt
	s.publishProjectEventAsync(ctx, id, projectdomain.NewProjectDeletedEvent(project))
	return nil
}

func normalizeDeleteInput(inputs []DeleteInput) DeleteInput {
	if len(inputs) == 0 {
		return DeleteInput{}
	}
	return inputs[0]
}

func validateDeleteConfirmation(project projectdomain.Project, input DeleteInput) error {
	confirmation := strings.TrimSpace(input.Confirmation)
	if confirmation == "" {
		return apperror.BadRequest("project delete confirmation is required", oops.In("project").With("project_id", project.ID, "full_path", project.FullPath).New("project delete confirmation is required"))
	}
	if confirmation != project.FullPath {
		return apperror.BadRequest("project delete confirmation does not match full path", oops.In("project").With("project_id", project.ID, "full_path", project.FullPath, "confirmation", confirmation).New("project delete confirmation does not match full path"))
	}
	return nil
}
