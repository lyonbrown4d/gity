package pipeline

import (
	"context"
	"errors"
	"time"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/samber/oops"
)

func (s *Service) loadPipeline(ctx context.Context, projectID, pipelineID int64) (cidomain.ProjectPipeline, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectPipeline{}, apperror.NotFound("project not found", err)
	}
	item, err := s.pipelineRepo.GetByProjectAndID(ctx, projectID, pipelineID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return cidomain.ProjectPipeline{}, apperror.NotFound("project pipeline not found", err)
		}
		return cidomain.ProjectPipeline{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "load pipeline")
	}
	return item, nil
}

func (s *Service) refreshPipelineState(ctx context.Context, projectID, pipelineID int64) (cidomain.ProjectPipeline, error) {
	pipeline, err := s.loadPipeline(ctx, projectID, pipelineID)
	if err != nil {
		return cidomain.ProjectPipeline{}, err
	}
	items, err := s.pipelineJobRepo.ListByPipelineID(ctx, projectID, pipelineID)
	if err != nil {
		return cidomain.ProjectPipeline{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "list pipeline jobs")
	}
	jobs := make(map[string]cidomain.ProjectJob, items.Len())
	itemValues := items.Values()
	for i := range itemValues {
		item := itemValues[i]
		job, jobErr := s.jobRepo.GetByID(ctx, item.ProjectJobID)
		if jobErr != nil {
			return cidomain.ProjectPipeline{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", item.ProjectJobID).Wrapf(jobErr, "load pipeline project job")
		}
		jobs[item.Stage] = job
	}
	nextStatus := pipelineStatus(jobs)
	if nextStatus != gitports.ProjectPipelineStatusFailed && nextStatus != gitports.ProjectPipelineStatusCancelled {
		if releaseErr := s.releaseReadyJobs(ctx, itemValues, jobs); releaseErr != nil {
			return cidomain.ProjectPipeline{}, releaseErr
		}
		nextStatus = pipelineStatus(jobs)
	}
	if nextStatus == gitports.ProjectPipelineStatusFailed || nextStatus == gitports.ProjectPipelineStatusCancelled {
		if cancelErr := s.cancelPendingJobs(ctx, jobs); cancelErr != nil {
			return cidomain.ProjectPipeline{}, cancelErr
		}
	}
	if updateErr := s.pipelineRepo.UpdateStatus(ctx, pipeline, nextStatus); updateErr != nil {
		return cidomain.ProjectPipeline{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "status", nextStatus).Wrapf(updateErr, "update pipeline status")
	}
	updated, err := s.pipelineRepo.GetByProjectAndID(ctx, projectID, pipelineID)
	if err != nil {
		return cidomain.ProjectPipeline{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "reload pipeline")
	}
	return updated, nil
}

func (s *Service) releaseReadyJobs(ctx context.Context, items []cidomain.ProjectPipelineJob, jobs map[string]cidomain.ProjectJob) error {
	now := time.Now().UTC()
	for i := range items {
		item := items[i]
		job := jobs[item.Stage]
		if job.Status != gitports.ProjectJobStatusPending || !isBlockedRunAfter(job.RunAfter) {
			continue
		}
		needs, err := decodeStringSlice(item.Needs)
		if err != nil {
			return err
		}
		if !needsSucceeded(needs, jobs) {
			continue
		}
		if err := s.jobRepo.ScheduleByID(ctx, job.ID, now); err != nil {
			return oops.In("pipeline").With("project_job_id", job.ID, "stage", item.Stage).Wrapf(err, "schedule pipeline job")
		}
		job.RunAfter = now
		job.UpdatedAt = now
		jobs[item.Stage] = job
	}
	return nil
}

func (s *Service) cancelPendingJobs(ctx context.Context, jobs map[string]cidomain.ProjectJob) error {
	for name := range jobs {
		job := jobs[name]
		if job.Status == gitports.ProjectJobStatusPending {
			if err := s.jobRepo.CancelByID(ctx, job.ID); err != nil {
				return oops.In("pipeline").With("project_job_id", job.ID).Wrapf(err, "cancel pending pipeline job")
			}
		}
	}
	return nil
}
