package runner

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"strings"
	"time"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/samber/oops"
)

const runnerOfflineAfter = 2 * time.Minute

func (s *Service) authenticateRunner(ctx context.Context, token string) (cidomain.ProjectRunner, error) {
	token = strings.TrimSpace(token)
	if token == "" {
		return cidomain.ProjectRunner{}, apperror.Unauthorized("runner token is required", oops.In("runner").New("runner token is required"))
	}
	runner, err := s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return cidomain.ProjectRunner{}, apperror.Unauthorized("invalid runner token", err)
		}
		return cidomain.ProjectRunner{}, oops.In("runner").Wrapf(err, "authenticate runner")
	}
	if runner.Active != 1 {
		return cidomain.ProjectRunner{}, apperror.Forbidden("runner is disabled", oops.In("runner").With("runner_id", runner.ID, "project_id", runner.ProjectID).New("runner is disabled"))
	}
	return runner, nil
}

func ensureRunnerOwnsJob(runner cidomain.ProjectRunner, job cidomain.ProjectJob) error {
	expected := runnerWorkerID(runner)
	if strings.TrimSpace(job.LockedBy) != expected {
		return apperror.Conflict("project job is not claimed by runner", fmt.Errorf("project job locked_by=%q expected=%q", job.LockedBy, expected))
	}
	return nil
}

func (s *Service) refreshPipelineForJob(ctx context.Context, projectID, jobID int64) error {
	if s.pipelineSvc == nil {
		return nil
	}
	if err := s.pipelineSvc.RefreshProjectJob(ctx, projectID, jobID); err != nil {
		return oops.In("runner").With("project_id", projectID, "job_id", jobID).Wrapf(err, "refresh pipeline for job")
	}
	return nil
}

func toRunnerView(item cidomain.ProjectRunner) RunnerView {
	now := time.Now().UTC()
	var lastContactAt *time.Time
	if !item.LastContactAt.IsZero() {
		value := item.LastContactAt
		lastContactAt = &value
	}
	return RunnerView{
		ID:            item.ID,
		ProjectID:     item.ProjectID,
		Name:          item.Name,
		Description:   item.Description,
		Tags:          item.Tags,
		Status:        effectiveRunnerStatus(item, now),
		Active:        item.Active == 1,
		LastContactAt: lastContactAt,
		CreatedAt:     item.CreatedAt,
		UpdatedAt:     item.UpdatedAt,
	}
}

func effectiveRunnerStatus(item cidomain.ProjectRunner, now time.Time) string {
	if item.Active != 1 {
		return gitports.ProjectRunnerStatusOffline
	}
	if item.LastContactAt.IsZero() {
		return gitports.ProjectRunnerStatusOffline
	}
	if now.Sub(item.LastContactAt) > runnerOfflineAfter {
		return gitports.ProjectRunnerStatusOffline
	}
	return gitports.ProjectRunnerStatusOnline
}

func runnerWorkerID(item cidomain.ProjectRunner) string {
	return fmt.Sprintf("runner:%d", item.ID)
}

func generateRunnerToken() (string, error) {
	var buffer [32]byte
	if _, err := rand.Read(buffer[:]); err != nil {
		return "", oops.In("runner").Wrapf(err, "generate runner token")
	}
	return "grt_" + base64.RawURLEncoding.EncodeToString(buffer[:]), nil
}
