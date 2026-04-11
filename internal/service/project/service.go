package project

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/httpx"
	"github.com/DaiYuANg/gity/internal/entity"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
)

type Service struct {
	logger        *slog.Logger
	repo          *projectrepo.Repository
	gitRunner     *gitexec.Runner
	gitRepository *gitrepo.Service
	namespaceRepo *namespacerepo.Repository
}

type CreateInput struct {
	NamespaceID   int64  `json:"namespace_id"`
	Name          string `json:"name"`
	PathKey       string `json:"path_key"`
	Visibility    string `json:"visibility"`
	Description   string `json:"description"`
	DefaultBranch string `json:"default_branch"`
}

func NewService(logger *slog.Logger, repo *projectrepo.Repository, gitRunner *gitexec.Runner, gitRepository *gitrepo.Service, namespaceRepo *namespacerepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, gitRunner: gitRunner, gitRepository: gitRepository, namespaceRepo: namespaceRepo}
}

func (s *Service) List(ctx context.Context, namespaceID *int64) (collectionx.List[entity.Project], error) {
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
	namespace, err := s.namespaceRepo.GetByID(ctx, input.NamespaceID)
	if err != nil {
		return entity.Project{}, fmt.Errorf("load project namespace: %w", err)
	}
	project, err := s.repo.Create(ctx, projectrepo.CreateInput{
		NamespaceID:   input.NamespaceID,
		Name:          input.Name,
		PathKey:       input.PathKey,
		Visibility:    input.Visibility,
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

func (s *Service) ListBranches(ctx context.Context, id int64) ([]gitrepo.Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	branches, err := s.gitRepository.ListBranches(ctx, repositoryPath(project), project.DefaultBranch)
	if err != nil {
		return nil, mapGitError(err)
	}
	return branches, nil
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

func repositoryPath(project entity.Project) string {
	return project.FullPath + ".git"
}

func mapGitError(err error) error {
	switch {
	case errors.Is(err, gitrepo.ErrRepositoryNotFound):
		return httpx.NewError(http.StatusNotFound, "repository not found", err)
	case errors.Is(err, gitrepo.ErrReferenceNotFound):
		return httpx.NewError(http.StatusNotFound, "git reference not found", err)
	case errors.Is(err, gitrepo.ErrPathNotFound):
		return httpx.NewError(http.StatusNotFound, "repository path not found", err)
	case errors.Is(err, gitrepo.ErrReadmeNotFound):
		return httpx.NewError(http.StatusNotFound, "repository readme not found", err)
	case errors.Is(err, gitrepo.ErrEmptyRepository):
		return httpx.NewError(http.StatusNotFound, "repository has no commits", err)
	default:
		return err
	}
}
