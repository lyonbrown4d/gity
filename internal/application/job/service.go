package job

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
	"log/slog"
	"strings"
	"time"
)

const (
	KindNoop   = "noop"
	KindScript = "script"
)

var supportedKinds = setx.NewSet(KindNoop, KindScript)

type Service struct {
	logger       *slog.Logger
	projectRepo  storageports.ProjectRepository
	jobRepo      storageports.ProjectJobRepository
	logRepo      storageports.ProjectJobLogRepository
	artifactRepo storageports.ProjectJobArtifactRepository
	storage      storageports.ObjectStorage
}

type CreateInput struct {
	Kind        string    `json:"kind"`
	Payload     string    `json:"payload"`
	MaxAttempts int       `json:"max_attempts"`
	RunAfter    time.Time `json:"run_after"`
}

type UploadArtifactInput struct {
	Name          string `json:"name"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

type AppendTraceInput struct {
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
}

type ProjectJobTrace struct {
	Job             cidomain.ProjectJob      `json:"job"`
	Logs            []cidomain.ProjectJobLog `json:"logs"`
	Trace           string                   `json:"trace"`
	ExitCode        int                      `json:"exit_code"`
	OutputTruncated bool                     `json:"output_truncated"`
	DurationMillis  int64                    `json:"duration_millis"`
}

type ProjectJobArtifactContent struct {
	Artifact      cidomain.ProjectJobArtifact `json:"artifact"`
	ContentBase64 string                      `json:"content_base64"`
}

type scriptResult struct {
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	WorkDir         string `json:"work_dir"`
}

func NewService(
	logger *slog.Logger,
	projectRepo storageports.ProjectRepository,
	jobRepo storageports.ProjectJobRepository,
	logRepo storageports.ProjectJobLogRepository,
	artifactRepo storageports.ProjectJobArtifactRepository,
	storage storageports.ObjectStorage,
) *Service {
	if logger == nil {
		logger = slog.Default()
	}
	return &Service{logger: logger, projectRepo: projectRepo, jobRepo: jobRepo, logRepo: logRepo, artifactRepo: artifactRepo, storage: storage}
}

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

func (s *Service) GetProjectJobTrace(ctx context.Context, projectID, jobID int64) (ProjectJobTrace, error) {
	job, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
	}
	logs, err := s.logRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "list project job trace")
	}
	trace := ProjectJobTrace{Job: job, Logs: logs.Values()}
	if logs.Len() == 0 {
		trace.Trace = fallbackTrace(job)
		return trace, nil
	}
	latest, _ := logs.Get(logs.Len() - 1)
	trace.Trace = latest.Output
	trace.ExitCode = latest.ExitCode
	trace.OutputTruncated = latest.OutputTruncated == 1
	trace.DurationMillis = latest.DurationMillis
	return trace, nil
}

func (s *Service) ListProjectJobArtifacts(ctx context.Context, projectID, jobID int64) ([]cidomain.ProjectJobArtifact, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return nil, err
	}
	items, err := s.artifactRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return nil, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "list project job artifacts")
	}
	return items.Values(), nil
}

func (s *Service) GetProjectJobArtifactContent(ctx context.Context, projectID, jobID, artifactID int64) (ProjectJobArtifactContent, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return ProjectJobArtifactContent{}, err
	}
	artifact, err := s.artifactRepo.GetByProjectJobAndID(ctx, projectID, jobID, artifactID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact not found", err)
		}
		return ProjectJobArtifactContent{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifactID).Wrapf(err, "get project job artifact")
	}
	content, err := s.storage.Load(ctx, artifact.StorageKey)
	if err != nil {
		return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact content not found", err)
	}
	return ProjectJobArtifactContent{Artifact: artifact, ContentBase64: base64.StdEncoding.EncodeToString(content)}, nil
}

func (s *Service) UploadProjectJobArtifact(ctx context.Context, projectID, jobID int64, input UploadArtifactInput) (cidomain.ProjectJobArtifact, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, apperror.NotFound("project not found", err)
	}
	if _, jobErr := s.GetProjectJob(ctx, projectID, jobID); jobErr != nil {
		return cidomain.ProjectJobArtifact{}, jobErr
	}
	fileName := strings.TrimSpace(input.FileName)
	if fileName == "" {
		return cidomain.ProjectJobArtifact{}, apperror.BadRequest("artifact file_name is required", oops.In("job").With("project_id", projectID, "job_id", jobID).New("artifact file_name is required"))
	}
	if strings.TrimSpace(input.ContentBase64) == "" {
		return cidomain.ProjectJobArtifact{}, apperror.BadRequest("artifact content_base64 is required", oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", fileName).New("artifact content_base64 is required"))
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", fileName).Wrapf(err, "decode artifact content")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(fileName)
	}
	artifact, err := s.artifactRepo.Create(ctx, storageports.CreateProjectJobArtifactInput{
		ProjectID:    projectID,
		ProjectJobID: jobID,
		Name:         input.Name,
		FileName:     fileName,
		FilePath:     input.FilePath,
		ContentType:  contentType,
	})
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", fileName).Wrapf(err, "create project job artifact")
	}
	storageKey, err := s.storage.SavePipelineArtifact(ctx, project.FullPath, jobID, artifact.ID, fileName, content, contentType)
	if err != nil {
		if cleanupErr := s.artifactRepo.DeleteByID(ctx, artifact.ID); cleanupErr != nil {
			return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(oops.Join(err, cleanupErr), "save project job artifact and cleanup record")
		}
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(err, "save project job artifact")
	}
	if storeErr := s.artifactRepo.MarkStored(ctx, artifact.ID, storageports.StoreProjectJobArtifactInput{ContentType: contentType, ByteSize: int64(len(content)), StorageKey: storageKey}); storeErr != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID, "storage_key", storageKey).Wrapf(storeErr, "mark project job artifact stored")
	}
	stored, err := s.artifactRepo.GetByID(ctx, artifact.ID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(err, "load stored project job artifact")
	}
	return stored, nil
}

func (s *Service) AppendProjectJobTrace(ctx context.Context, projectID, jobID int64, input AppendTraceInput) (ProjectJobTrace, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
	}
	if item.Kind != KindScript {
		return ProjectJobTrace{}, apperror.BadRequest("project job does not support trace streaming", fmt.Errorf("project job kind: %s", item.Kind))
	}
	if item.Status != storageports.ProjectJobStatusRunning {
		return ProjectJobTrace{}, apperror.Conflict("project job is not running", fmt.Errorf("project job status: %s", item.Status))
	}
	if s.logRepo == nil {
		return ProjectJobTrace{}, apperror.Internal("project job log repository is not configured")
	}
	if input.Output == "" && !input.OutputTruncated {
		return s.GetProjectJobTrace(ctx, projectID, jobID)
	}
	if _, err := s.logRepo.Append(ctx, storageports.AppendProjectJobLogInput{
		ProjectID:       projectID,
		ProjectJobID:    jobID,
		Attempt:         item.Attempts,
		Output:          input.Output,
		OutputTruncated: input.OutputTruncated,
		DurationMillis:  input.DurationMillis,
	}); err != nil {
		return ProjectJobTrace{}, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "append project job trace")
	}
	return s.GetProjectJobTrace(ctx, projectID, jobID)
}

func (s *Service) RunNext(ctx context.Context, workerID string, lease time.Duration) (bool, error) {
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
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, false, apperror.NotFound("project not found", err)
	}
	job, claimed, err := s.jobRepo.ClaimNextByProjectIDAndKinds(ctx, projectID, []string{KindScript}, normalizeWorkerID(workerID), lease)
	if err != nil {
		return cidomain.ProjectJob{}, false, oops.In("job").With("project_id", projectID, "worker_id", normalizeWorkerID(workerID)).Wrapf(err, "claim project script job")
	}
	return job, claimed, nil
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

func (s *Service) recordScriptLog(ctx context.Context, item cidomain.ProjectJob, result, fallback string) error {
	if item.Kind != KindScript || s.logRepo == nil {
		return nil
	}
	parsed := parseScriptResult(result, fallback)
	_, err := s.logRepo.UpsertAttempt(ctx, storageports.CreateProjectJobLogInput{
		ProjectID:       item.ProjectID,
		ProjectJobID:    item.ID,
		Attempt:         item.Attempts,
		ExitCode:        parsed.ExitCode,
		Output:          parsed.Output,
		OutputTruncated: parsed.OutputTruncated,
		DurationMillis:  parsed.DurationMillis,
	})
	if err != nil {
		return oops.In("job").With("project_id", item.ProjectID, "job_id", item.ID, "attempt", item.Attempts).Wrapf(err, "record project job script log")
	}
	return nil
}

func parseScriptResult(result, fallback string) scriptResult {
	trimmed := strings.TrimSpace(result)
	if trimmed == "" {
		return fallbackScriptResult(fallback)
	}
	if parsed, ok := decodeScriptResult(trimmed); ok {
		return parsed
	}
	return scriptResult{Output: trimmed}
}

func fallbackScriptResult(fallback string) scriptResult {
	parsed := scriptResult{Output: strings.TrimSpace(fallback)}
	if parsed.Output != "" {
		parsed.ExitCode = 1
	}
	return parsed
}

func decodeScriptResult(trimmed string) (scriptResult, bool) {
	var parsed scriptResult
	if err := json.Unmarshal([]byte(trimmed), &parsed); err == nil {
		return parsed, true
	}
	index := strings.LastIndex(trimmed, "\n{")
	if index < 0 {
		return scriptResult{}, false
	}
	if err := json.Unmarshal([]byte(strings.TrimSpace(trimmed[index+1:])), &parsed); err != nil {
		return scriptResult{}, false
	}
	if prefix := strings.TrimSpace(trimmed[:index]); prefix != "" && parsed.Output == "" {
		parsed.Output = prefix
	}
	return parsed, true
}

func fallbackTrace(job cidomain.ProjectJob) string {
	if strings.TrimSpace(job.Result) != "" {
		return parseScriptResult(job.Result, "").Output
	}
	return strings.TrimSpace(job.LastError)
}

func retryDelay(attempts int) time.Duration {
	if attempts <= 0 {
		return 5 * time.Second
	}
	delay := time.Duration(attempts*5) * time.Second
	if delay > time.Minute {
		return time.Minute
	}
	return delay
}

func normalizeWorkerID(workerID string) string {
	trimmed := strings.TrimSpace(workerID)
	if trimmed == "" {
		return "worker"
	}
	return trimmed
}
