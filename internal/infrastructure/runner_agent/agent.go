// Package runneragent implements the standalone runner agent.
package runneragent

import (
	"context"
	"log/slog"
	"strings"
	"sync/atomic"
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/samber/oops"
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
			return oops.In("runner_agent").Wrap(ctx.Err())
		case <-ticker.C:
		}
	}
}

func (a *Agent) RunOnce(ctx context.Context) (bool, error) {
	claim, err := a.client.ClaimJob(ctx, a.cfg.LeaseSeconds)
	if err != nil {
		return false, oops.In("runner_agent").Wrapf(err, "claim runner job")
	}
	if !claim.Claimed {
		return false, nil
	}
	a.logger.Info("runner claimed job", slog.Int64("job_id", claim.Job.ID), slog.String("kind", claim.Job.Kind))
	if err := a.executeClaimedJob(ctx, claim.Job); err != nil {
		return true, oops.In("runner_agent").With("project_id", claim.Job.ProjectID, "job_id", claim.Job.ID).Wrapf(err, "execute claimed job")
	}
	return true, nil
}

func (a *Agent) executeClaimedJob(ctx context.Context, job cidomain.ProjectJob) error {
	if job.Kind == "script" {
		return a.executeScriptJob(ctx, job)
	}
	return a.failUnsupportedJob(ctx, job)
}

func (a *Agent) executeScriptJob(ctx context.Context, job cidomain.ProjectJob) error {
	callbacks := a.scriptCallbacks(job)
	result, err := ExecuteScriptJobWithSource(ctx, a.cfg, job, callbacks.checker, callbacks.trace, callbacks.source)
	artifactErr := a.uploadArtifacts(ctx, job, result)
	if err != nil {
		return a.reportScriptFailure(ctx, job, result, err, artifactErr)
	}
	if artifactErr != nil {
		return a.reportArtifactFailure(ctx, job, result, artifactErr)
	}
	if err := a.client.CompleteJob(ctx, job.ID, result); err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "complete runner job")
	}
	a.logger.Info("runner job completed", slog.Int64("job_id", job.ID))
	return nil
}

type scriptCallbacks struct {
	checker ScriptCancellationChecker
	trace   ScriptTraceStreamer
	source  ScriptSourceFetcher
}

func (a *Agent) scriptCallbacks(job cidomain.ProjectJob) scriptCallbacks {
	var traceWarned atomic.Bool
	return scriptCallbacks{
		checker: a.scriptCancellationChecker(job),
		trace:   a.scriptTraceStreamer(job, &traceWarned),
		source:  a.scriptSourceFetcher(),
	}
}

func (a *Agent) scriptCancellationChecker(job cidomain.ProjectJob) ScriptCancellationChecker {
	expectedLocker := job.LockedBy
	return func(checkCtx context.Context) (bool, error) {
		current, err := a.client.GetProjectJob(checkCtx, job.ProjectID, job.ID)
		if err != nil {
			return false, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "check claimed job status")
		}
		return strings.TrimSpace(current.LockedBy) != expectedLocker || strings.TrimSpace(current.Status) != runningJobStatus, nil
	}
}

func (a *Agent) scriptTraceStreamer(job cidomain.ProjectJob, warned *atomic.Bool) ScriptTraceStreamer {
	return func(traceCtx context.Context, output string, outputTruncated bool, durationMillis int64) error {
		if err := a.client.AppendTrace(traceCtx, job.ID, output, outputTruncated, durationMillis); err != nil {
			if warned.CompareAndSwap(false, true) {
				a.logger.Warn("runner trace upload failed", slog.Int64("job_id", job.ID), slog.String("error", err.Error()))
			}
		}
		return nil
	}
}

func (a *Agent) scriptSourceFetcher() ScriptSourceFetcher {
	return func(sourceCtx context.Context, job cidomain.ProjectJob, _ ScriptPayload, workDir string) error {
		content, err := a.client.DownloadSourceArchive(sourceCtx, job.ID)
		if err != nil {
			return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "download source archive")
		}
		return ExtractSourceArchive(content, workDir)
	}
}

func (a *Agent) reportScriptFailure(ctx context.Context, job cidomain.ProjectJob, result string, err, artifactErr error) error {
	reportErr := a.client.FailJob(ctx, job.ID, err.Error(), result, 0)
	if reportErr != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(oops.Join(reportErr, err), "report script job failure")
	}
	if artifactErr != nil {
		a.logger.Warn("runner artifact upload failed", slog.Int64("job_id", job.ID), slog.String("error", artifactErr.Error()))
	}
	a.logger.Warn("runner job failed", slog.Int64("job_id", job.ID), slog.String("error", err.Error()))
	return nil
}

func (a *Agent) reportArtifactFailure(ctx context.Context, job cidomain.ProjectJob, result string, artifactErr error) error {
	reportErr := a.client.FailJob(ctx, job.ID, "artifact upload failed: "+artifactErr.Error(), result, 0)
	if reportErr != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(oops.Join(reportErr, artifactErr), "report artifact upload failure")
	}
	a.logger.Warn("runner artifact upload failed", slog.Int64("job_id", job.ID), slog.String("error", artifactErr.Error()))
	return nil
}

func (a *Agent) failUnsupportedJob(ctx context.Context, job cidomain.ProjectJob) error {
	message := "runner does not support job kind: " + job.Kind
	if err := a.client.FailJob(ctx, job.ID, message, "", 0); err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "kind", job.Kind).Wrapf(err, "report unsupported runner job kind")
	}
	return nil
}

func (a *Agent) uploadArtifacts(ctx context.Context, job cidomain.ProjectJob, result string) error {
	if strings.TrimSpace(result) == "" {
		return nil
	}
	artifacts, err := CollectArtifacts(job, result)
	if err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "collect runner artifacts")
	}
	for _, artifact := range artifacts {
		if err := a.client.UploadArtifact(ctx, job.ID, artifact); err != nil {
			return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "artifact_path", artifact.FilePath).Wrapf(err, "upload runner artifact")
		}
		a.logger.Info("runner artifact uploaded", slog.Int64("job_id", job.ID), slog.String("path", artifact.FilePath))
	}
	return nil
}
