package runner

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

func (s *Service) DownloadSourceArchive(ctx context.Context, token string, jobID int64) (SourceArchiveView, error) {
	runner, job, err := s.loadSourceArchiveJob(ctx, token, jobID)
	if err != nil {
		return SourceArchiveView{}, err
	}
	if s.gitRunner == nil {
		return SourceArchiveView{}, apperror.Internal("git runner is not configured")
	}
	project, err := s.projectRepo.GetByID(ctx, runner.ProjectID)
	if err != nil {
		return SourceArchiveView{}, apperror.NotFound("project not found", err)
	}
	payload, err := decodeScriptSourcePayload(job)
	if err != nil {
		return SourceArchiveView{}, err
	}
	revision, err := resolveSourceArchiveRevision(runner.ProjectID, jobID, project.FullPath, payload)
	if err != nil {
		return SourceArchiveView{}, err
	}
	if heartbeatErr := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); heartbeatErr != nil {
		return SourceArchiveView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID).Wrapf(heartbeatErr, "mark runner heartbeat")
	}
	content, err := s.gitRunner.Archive(ctx, project.FullPath+".git", revision)
	if err != nil {
		return SourceArchiveView{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID, "revision", revision).Wrapf(err, "archive project source")
	}
	return SourceArchiveView{
		FileName:      fmt.Sprintf("project-%d-job-%d-source.zip", project.ID, job.ID),
		Encoding:      "base64",
		ContentBase64: base64.StdEncoding.EncodeToString(content),
	}, nil
}

func (s *Service) loadSourceArchiveJob(ctx context.Context, token string, jobID int64) (cidomain.ProjectRunner, cidomain.ProjectJob, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectRunner{}, cidomain.ProjectJob{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectRunner{}, cidomain.ProjectJob{}, oops.In("runner").With("project_id", runner.ProjectID, "runner_id", runner.ID, "job_id", jobID).Wrapf(err, "load project job")
	}
	if ownershipErr := ensureRunnerCanUseJob(runner, job, time.Now().UTC()); ownershipErr != nil {
		return cidomain.ProjectRunner{}, cidomain.ProjectJob{}, ownershipErr
	}
	return runner, job, nil
}

func resolveSourceArchiveRevision(projectID, jobID int64, projectFullPath string, payload scriptSourcePayload) (string, error) {
	if payload.ProjectFullPath != "" && payload.ProjectFullPath != projectFullPath {
		return "", apperror.BadRequest("job project path does not match runner project", fmt.Errorf("payload project=%q expected=%q", payload.ProjectFullPath, projectFullPath))
	}
	revision := firstNonEmpty(payload.CommitSHA, payload.RefName)
	if revision == "" {
		return "", apperror.BadRequest("job source revision is required", oops.In("runner").With("project_id", projectID, "job_id", jobID).New("job source revision is required"))
	}
	return revision, nil
}

func decodeScriptSourcePayload(job cidomain.ProjectJob) (scriptSourcePayload, error) {
	if strings.TrimSpace(job.Kind) != jobservice.KindScript {
		return scriptSourcePayload{}, apperror.BadRequest("job source archive is only available for script jobs", fmt.Errorf("job kind: %s", job.Kind))
	}
	var payload scriptSourcePayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return scriptSourcePayload{}, apperror.BadRequest("invalid script job payload", err)
	}
	payload.ProjectFullPath = strings.Trim(strings.ReplaceAll(payload.ProjectFullPath, "\\", "/"), "/")
	payload.RefName = strings.TrimSpace(payload.RefName)
	payload.CommitSHA = strings.TrimSpace(payload.CommitSHA)
	return payload, nil
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return strings.TrimSpace(value)
		}
	}
	return ""
}
