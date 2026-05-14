package mergerequest

import (
	"context"
	"errors"
	"log/slog"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

func (s *Service) publishEventAsync(ctx context.Context, event mergedomain.ProjectMergeRequestMerged) {
	if err := s.events.PublishAsync(ctx, event); err != nil {
		wrapped := oops.In("merge_request").With("project_id", event.ProjectID, "merge_request_id", event.MergeRequestID, "merge_iid", event.MergeIID, "event", event.Name()).Wrapf(err, "publish merge request event")
		s.warn("publish merge request event failed", slog.String("event", event.Name()), slog.String("error", wrapped.Error()))
	}
}

func (s *Service) publishRepositoryChanged(ctx context.Context, project projectdomain.Project, branchName string) {
	event := projectdomain.NewProjectRepositoryChangedEvent(project, branchName, "", false, "merge_request")
	if err := s.events.PublishAsync(ctx, event); err != nil {
		wrapped := oops.In("merge_request").With("project_id", project.ID, "branch", branchName, "event", event.Name()).Wrapf(err, "publish repository changed event")
		s.warn("publish repository changed event failed", slog.String("event", event.Name()), slog.String("error", wrapped.Error()))
	}
}

func (s *Service) loadMergeRequest(ctx context.Context, projectID, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	_, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	return mr, err
}

func (s *Service) loadProjectMergeRequest(ctx context.Context, projectID, mergeIID int64) (projectdomain.Project, mergedomain.ProjectMergeRequest, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, mergedomain.ProjectMergeRequest{}, apperror.NotFound("project not found", err)
	}
	mr, err := s.loadMergeRequestRecord(ctx, projectID, mergeIID)
	if err != nil {
		return projectdomain.Project{}, mergedomain.ProjectMergeRequest{}, err
	}
	return project, mr, nil
}

func (s *Service) loadMergeRequestRecord(ctx context.Context, projectID, mergeIID int64) (mergedomain.ProjectMergeRequest, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return mergedomain.ProjectMergeRequest{}, apperror.NotFound("project not found", err)
	}
	mr, err := s.mergeRepo.GetByProjectAndIID(ctx, projectID, mergeIID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return mergedomain.ProjectMergeRequest{}, apperror.NotFound("merge request not found", err)
		}
		return mergedomain.ProjectMergeRequest{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).Wrapf(err, "load merge request")
	}
	return mr, nil
}
