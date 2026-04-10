package project

import (
	"context"
	"database/sql"
	"fmt"
	"log/slog"
	"strings"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/gity/internal/entity"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
)

type Service struct {
	logger        *slog.Logger
	repo          *projectrepo.Repository
	gitRunner     *gitexec.Runner
	namespaceRepo *namespacerepo.Repository
}

type CreateInput struct {
	NamespaceID   int64  `json:"namespace_id"`
	Name          string `json:"name"`
	PathKey       string `json:"path_key"`
	Description   string `json:"description"`
	DefaultBranch string `json:"default_branch"`
}

func NewService(logger *slog.Logger, repo *projectrepo.Repository, gitRunner *gitexec.Runner, namespaceRepo *namespacerepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, gitRunner: gitRunner, namespaceRepo: namespaceRepo}
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
