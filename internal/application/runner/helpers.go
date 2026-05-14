package runner

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	setx "github.com/arcgolabs/collectionx/set"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
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

func ensureRunnerCanUseJob(runner cidomain.ProjectRunner, job cidomain.ProjectJob, now time.Time) error {
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return err
	}
	if strings.TrimSpace(job.Status) != gitports.ProjectJobStatusRunning {
		return apperror.Conflict("project job is not running", fmt.Errorf("project job status=%q", job.Status))
	}
	if job.LockedUntil.IsZero() || !job.LockedUntil.After(now.UTC()) {
		return apperror.Conflict("project job lease has expired", fmt.Errorf("project job locked_until=%s", job.LockedUntil.UTC().Format(time.RFC3339Nano)))
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

func runnerMatchesJob(runner cidomain.ProjectRunner, job cidomain.ProjectJob) (bool, error) {
	requiredTags, err := scriptJobTags(job.Payload)
	if err != nil {
		return false, err
	}
	if len(requiredTags) == 0 {
		return true, nil
	}
	runnerTags := tagSet(runner.Tags)
	for _, tag := range requiredTags {
		if !runnerTags.Contains(tag) {
			return false, nil
		}
	}
	return true, nil
}

func scriptJobTags(payload string) ([]string, error) {
	if strings.TrimSpace(payload) == "" {
		return nil, nil
	}
	var out struct {
		Tags []string `json:"tags"`
	}
	if err := json.Unmarshal([]byte(payload), &out); err != nil {
		return nil, fmt.Errorf("decode script job tags: %w", err)
	}
	tags := tagSet(strings.Join(out.Tags, ","))
	return tags.Values(), nil
}

func tagSet(value string) *setx.Set[string] {
	parts := strings.Split(value, ",")
	tags := setx.NewSetWithCapacity[string](len(parts))
	for _, part := range parts {
		trimmed := strings.TrimSpace(strings.ToLower(part))
		if trimmed != "" {
			tags.Add(trimmed)
		}
	}
	return tags
}

func generateRunnerToken() (string, error) {
	var buffer [32]byte
	if _, err := rand.Read(buffer[:]); err != nil {
		return "", oops.In("runner").Wrapf(err, "generate runner token")
	}
	return "grt_" + base64.RawURLEncoding.EncodeToString(buffer[:]), nil
}
