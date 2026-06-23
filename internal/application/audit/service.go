package audit

import (
	"context"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	auditports "github.com/lyonbrown4d/gity/internal/application/ports"
	auditdomain "github.com/lyonbrown4d/gity/internal/domain/audit"
	"github.com/samber/oops"
)

type Service struct {
	repo        auditports.ProjectAuditEventRepository
	projectRepo auditports.ProjectRepository
}

type Dependencies struct {
	Repo        auditports.ProjectAuditEventRepository
	ProjectRepo auditports.ProjectRepository
}

func NewDependencies(repo auditports.ProjectAuditEventRepository, projectRepo auditports.ProjectRepository) Dependencies {
	return Dependencies{Repo: repo, ProjectRepo: projectRepo}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{repo: dependencies.Repo, projectRepo: dependencies.ProjectRepo}
}

func NewService(repo auditports.ProjectAuditEventRepository, projectRepo auditports.ProjectRepository) *Service {
	return NewServiceWithDependencies(NewDependencies(repo, projectRepo))
}

func (s *Service) ListProjectEvents(ctx context.Context, projectID int64, limit int) ([]auditdomain.ProjectAuditEvent, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.repo.ListByProjectID(ctx, projectID, limit)
	if err != nil {
		return nil, oops.In("audit").With("project_id", projectID, "limit", limit).Wrapf(err, "list project audit events")
	}
	return items.Values(), nil
}
