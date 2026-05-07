package runneragent

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"sync/atomic"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"
)

const (
	runningJobStatus = "running"
)

type Agent struct {
	cfg    Config
	client *Client
	logger *slog.Logger
}

func New(cfg Config, logger *slog.Logger) *Agent {
	if logger == nil {
		logger = slog.Default()
	}
	return &Agent{
		cfg:    cfg,
		client: NewClient(cfg.ServerURL, cfg.Token),
		logger: logger,
	}
}

func (a *Agent) Run(ctx context.Context) error {
	if err := a.client.Heartbeat(ctx); err != nil {
		return err
	}
	a.logger.Info("runner agent started", slog.String("server", a.cfg.ServerURL), slog.String("workdir", a.cfg.WorkDir))
	ticker := time.NewTicker(a.cfg.PollInterval)
	defer ticker.Stop()
	for {
		if _, err := a.RunOnce(ctx); err != nil {
			a.logger.Error("runner iteration failed", slog.String("error", err.Error()))
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-ticker.C:
		}
	}
}

func (a *Agent) RunOnce(ctx context.Context) (bool, error) {
	claim, err := a.client.ClaimJob(ctx, a.cfg.LeaseSeconds)
	if err != nil {
		return false, err
	}
	if !claim.Claimed {
		return false, nil
	}
	a.logger.Info("runner claimed job", slog.Int64("job_id", claim.Job.ID), slog.String("kind", claim.Job.Kind))
	if err := a.executeClaimedJob(ctx, claim.Job); err != nil {
		return true, err
	}
	return true, nil
}

func (a *Agent) executeClaimedJob(ctx context.Context, job entity.ProjectJob) error {
	expectedLocker := job.LockedBy
	switch job.Kind {
	case "script":
		var traceWarned atomic.Bool
		result, err := ExecuteScriptJobWithSource(ctx, a.cfg, job, func(checkCtx context.Context) (bool, error) {
			current, err := a.client.GetProjectJob(checkCtx, job.ProjectID, job.ID)
			if err != nil {
				return false, nil
			}
			if strings.TrimSpace(current.LockedBy) != expectedLocker {
				return true, nil
			}
			if strings.TrimSpace(current.Status) != runningJobStatus {
				return true, nil
			}
			return false, nil
		}, func(traceCtx context.Context, output string, outputTruncated bool, durationMillis int64) error {
			if err := a.client.AppendTrace(traceCtx, job.ID, output, outputTruncated, durationMillis); err != nil {
				if traceWarned.CompareAndSwap(false, true) {
					a.logger.Warn("runner trace upload failed", slog.Int64("job_id", job.ID), slog.String("error", err.Error()))
				}
				return nil
			}
			return nil
		}, func(sourceCtx context.Context, job entity.ProjectJob, _ ScriptPayload, workDir string) error {
			content, err := a.client.DownloadSourceArchive(sourceCtx, job.ID)
			if err != nil {
				return err
			}
			return ExtractSourceArchive(content, workDir)
		})
		artifactErr := a.uploadArtifacts(ctx, job, result)
		if err != nil {
			reportErr := a.client.FailJob(ctx, job.ID, err.Error(), result, 0)
			if reportErr != nil {
				return fmt.Errorf("report script job failure: %w; execution error: %v", reportErr, err)
			}
			if artifactErr != nil {
				a.logger.Warn("runner artifact upload failed", slog.Int64("job_id", job.ID), slog.String("error", artifactErr.Error()))
			}
			a.logger.Warn("runner job failed", slog.Int64("job_id", job.ID), slog.String("error", err.Error()))
			return nil
		}
		if artifactErr != nil {
			reportErr := a.client.FailJob(ctx, job.ID, "artifact upload failed: "+artifactErr.Error(), result, 0)
			if reportErr != nil {
				return fmt.Errorf("report artifact upload failure: %w; artifact error: %v", reportErr, artifactErr)
			}
			a.logger.Warn("runner artifact upload failed", slog.Int64("job_id", job.ID), slog.String("error", artifactErr.Error()))
			return nil
		}
		if err := a.client.CompleteJob(ctx, job.ID, result); err != nil {
			return err
		}
		a.logger.Info("runner job completed", slog.Int64("job_id", job.ID))
		return nil
	default:
		message := fmt.Sprintf("runner does not support job kind: %s", job.Kind)
		if err := a.client.FailJob(ctx, job.ID, message, "", 0); err != nil {
			return err
		}
		return nil
	}
}

func (a *Agent) uploadArtifacts(ctx context.Context, job entity.ProjectJob, result string) error {
	if strings.TrimSpace(result) == "" {
		return nil
	}
	artifacts, err := CollectArtifacts(job, result)
	if err != nil {
		return err
	}
	for _, artifact := range artifacts {
		if err := a.client.UploadArtifact(ctx, job.ID, artifact); err != nil {
			return err
		}
		a.logger.Info("runner artifact uploaded", slog.Int64("job_id", job.ID), slog.String("path", artifact.FilePath))
	}
	return nil
}
