package job

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"strings"
	"time"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

func (s *Service) ListProjectJobs(ctx context.Context, projectID int64) ([]cidomain.ProjectJob, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.jobRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("job").With("project_id", projectID).Wrapf(err, "list project jobs")
	}
	return items.Values(), nil
}

func (s *Service) GetProjectJob(ctx context.Context, projectID, jobID int64) (cidomain.ProjectJob, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, apperror.NotFound("project not found", err)
	}
	item, err := s.jobRepo.GetByProjectAndID(ctx, projectID, jobID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return cidomain.ProjectJob{}, apperror.NotFound("project job not found", err)
		}
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "get project job")
	}
	return item, nil
}

func (s *Service) EnqueueProjectJob(ctx context.Context, projectID int64, input CreateInput) (cidomain.ProjectJob, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, apperror.NotFound("project not found", err)
	}
	kind := strings.TrimSpace(strings.ToLower(input.Kind))
	if kind == "" {
		kind = KindNoop
	}
	if !supportedKinds.Contains(kind) {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "kind", kind).New("unsupported project job kind")
	}
	maxAttempts := input.MaxAttempts
	if maxAttempts <= 0 {
		maxAttempts = 3
	}
	item, err := s.jobRepo.Create(ctx, storageports.CreateProjectJobInput{
		ProjectID:   projectID,
		Kind:        kind,
		Payload:     input.Payload,
		MaxAttempts: maxAttempts,
		RunAfter:    input.RunAfter,
	})
	if err != nil {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "kind", kind).Wrapf(err, "enqueue project job")
	}
	return item, nil
}

func (s *Service) CancelProjectJob(ctx context.Context, projectID, jobID int64) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	switch item.Status {
	case storageports.ProjectJobStatusSucceeded, storageports.ProjectJobStatusFailed, storageports.ProjectJobStatusCancelled:
		return item, nil
	}
	if err := s.jobRepo.CancelByID(ctx, item.ID); err != nil {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "job_id", item.ID).Wrapf(err, "cancel project job")
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) RetryProjectJob(ctx context.Context, projectID, jobID int64) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if item.Status == storageports.ProjectJobStatusRunning {
		return cidomain.ProjectJob{}, apperror.Conflict("project job is running", fmt.Errorf("project job is running: %s", item.Status))
	}
	if item.Status == storageports.ProjectJobStatusSucceeded {
		return item, nil
	}
	if err := s.jobRepo.RetryByID(ctx, item.ID, time.Now().UTC()); err != nil {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "job_id", item.ID).Wrapf(err, "retry project job")
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) RunNext(ctx context.Context, workerID string, lease time.Duration) (bool, error) {
	if _, err := s.RequeueExpiredProjectJobs(ctx); err != nil {
		return false, err
	}
	job, claimed, err := s.jobRepo.ClaimNextByKinds(ctx, []string{KindNoop}, normalizeWorkerID(workerID), lease)
	if err != nil {
		return claimed, oops.In("job").With("worker_id", normalizeWorkerID(workerID)).Wrapf(err, "claim next project job")
	}
	if !claimed {
		return claimed, nil
	}
	result, execErr := s.execute(ctx, job)
	if execErr != nil {
		if err := s.jobRepo.MarkFailed(ctx, job, execErr.Error(), retryDelay(job.Attempts)); err != nil {
			return true, oops.In("job").With("project_id", job.ProjectID, "job_id", job.ID, "execution_error", execErr.Error()).Wrapf(err, "record project job failure after execution error")
		}
		s.logger.Warn("project job failed", slog.Int64("job_id", job.ID), slog.String("kind", job.Kind), slog.String("error", execErr.Error()))
		return true, nil
	}
	if err := s.jobRepo.MarkSucceeded(ctx, job.ID, result); err != nil {
		return true, oops.In("job").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "mark project job succeeded")
	}
	s.logger.Info("project job completed", slog.Int64("job_id", job.ID), slog.String("kind", job.Kind))
	return true, nil
}

func (s *Service) ClaimProjectJob(ctx context.Context, projectID int64, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	return s.ClaimProjectJobMatching(ctx, projectID, workerID, lease, func(cidomain.ProjectJob) (bool, error) {
		return true, nil
	})
}

func (s *Service) ClaimProjectJobMatching(ctx context.Context, projectID int64, workerID string, lease time.Duration, matcher ClaimMatcher) (cidomain.ProjectJob, bool, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, false, apperror.NotFound("project not found", err)
	}
	if _, err := s.RequeueExpiredProjectJobs(ctx); err != nil {
		return cidomain.ProjectJob{}, false, err
	}
	workerID = normalizeWorkerID(workerID)
	candidates, err := s.jobRepo.ListClaimableByProjectIDAndKinds(ctx, projectID, []string{KindScript}, 50)
	if err != nil {
		return cidomain.ProjectJob{}, false, oops.In("job").With("project_id", projectID, "worker_id", workerID).Wrapf(err, "list claimable project script jobs")
	}
	return s.claimMatchingCandidate(ctx, projectID, workerID, lease, normalizeClaimMatcher(matcher), candidates.Values())
}

func normalizeClaimMatcher(matcher ClaimMatcher) ClaimMatcher {
	if matcher != nil {
		return matcher
	}
	return func(cidomain.ProjectJob) (bool, error) { return true, nil }
}

func (s *Service) claimMatchingCandidate(ctx context.Context, projectID int64, workerID string, lease time.Duration, matcher ClaimMatcher, candidates []cidomain.ProjectJob) (cidomain.ProjectJob, bool, error) {
	for index := range candidates {
		candidate := candidates[index]
		matched, matchErr := matcher(candidate)
		if matchErr != nil {
			return cidomain.ProjectJob{}, false, oops.In("job").With("project_id", projectID, "job_id", candidate.ID, "worker_id", workerID).Wrapf(matchErr, "match claimable project job")
		}
		if !matched {
			continue
		}
		job, claimed, claimErr := s.jobRepo.ClaimByID(ctx, candidate.ID, workerID, lease)
		if claimErr != nil {
			return cidomain.ProjectJob{}, false, oops.In("job").With("project_id", projectID, "job_id", candidate.ID, "worker_id", workerID).Wrapf(claimErr, "claim project script job")
		}
		if claimed {
			return job, true, nil
		}
	}
	return cidomain.ProjectJob{}, false, nil
}

func (s *Service) RequeueExpiredProjectJobs(ctx context.Context) (int64, error) {
	expired, err := s.jobRepo.RequeueExpiredLeases(ctx, time.Now().UTC())
	if err != nil {
		return expired, oops.In("job").Wrapf(err, "requeue expired project job leases")
	}
	if expired > 0 && s.logger != nil {
		s.logger.Warn("expired project job leases requeued", slog.Int64("count", expired))
	}
	return expired, nil
}

func (s *Service) CompleteProjectJob(ctx context.Context, projectID, jobID int64, result string) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if item.Status != storageports.ProjectJobStatusRunning {
		return cidomain.ProjectJob{}, apperror.Conflict("project job is not running", fmt.Errorf("project job is not running: %s", item.Status))
	}
	if err := s.jobRepo.MarkSucceeded(ctx, item.ID, result); err != nil {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "job_id", item.ID).Wrapf(err, "complete project job")
	}
	if err := s.recordScriptLog(ctx, item, result, ""); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) FailProjectJob(ctx context.Context, projectID, jobID int64, message string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
	return s.FailProjectJobWithResult(ctx, projectID, jobID, message, "", retryAfter)
}

func (s *Service) FailProjectJobWithResult(ctx context.Context, projectID, jobID int64, message, result string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if item.Status != storageports.ProjectJobStatusRunning {
		return cidomain.ProjectJob{}, apperror.Conflict("project job is not running", fmt.Errorf("project job is not running: %s", item.Status))
	}
	if retryAfter <= 0 {
		retryAfter = retryDelay(item.Attempts)
	}
	if err := s.jobRepo.MarkFailed(ctx, item, message, retryAfter); err != nil {
		return cidomain.ProjectJob{}, oops.In("job").With("project_id", projectID, "job_id", item.ID).Wrapf(err, "fail project job")
	}
	if err := s.recordScriptLog(ctx, item, result, message); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) execute(ctx context.Context, item cidomain.ProjectJob) (string, error) {
	_ = ctx
	switch item.Kind {
	case KindNoop:
		return `{"message":"noop completed"}`, nil
	default:
		return "", oops.In("job").With("project_id", item.ProjectID, "job_id", item.ID, "kind", item.Kind).New("unsupported project job kind")
	}
}
