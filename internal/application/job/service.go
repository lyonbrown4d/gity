package job

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	setx "github.com/arcgolabs/collectionx/set"
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
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetProjectJob(ctx context.Context, projectID int64, jobID int64) (cidomain.ProjectJob, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, apperror.NotFound("project not found", err)
	}
	item, err := s.jobRepo.GetByProjectAndID(ctx, projectID, jobID)
	if err != nil {
		if err == storageports.ErrNotFound {
			return cidomain.ProjectJob{}, apperror.NotFound("project job not found", err)
		}
		return cidomain.ProjectJob{}, err
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
		return cidomain.ProjectJob{}, fmt.Errorf("unsupported project job kind: %s", kind)
	}
	maxAttempts := input.MaxAttempts
	if maxAttempts <= 0 {
		maxAttempts = 3
	}
	return s.jobRepo.Create(ctx, storageports.CreateProjectJobInput{
		ProjectID:   projectID,
		Kind:        kind,
		Payload:     input.Payload,
		MaxAttempts: maxAttempts,
		RunAfter:    input.RunAfter,
	})
}

func (s *Service) CancelProjectJob(ctx context.Context, projectID int64, jobID int64) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	switch item.Status {
	case storageports.ProjectJobStatusSucceeded, storageports.ProjectJobStatusFailed, storageports.ProjectJobStatusCancelled:
		return item, nil
	}
	if err := s.jobRepo.CancelByID(ctx, item.ID); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) RetryProjectJob(ctx context.Context, projectID int64, jobID int64) (cidomain.ProjectJob, error) {
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
		return cidomain.ProjectJob{}, err
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) GetProjectJobTrace(ctx context.Context, projectID int64, jobID int64) (ProjectJobTrace, error) {
	job, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
	}
	logs, err := s.logRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
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

func (s *Service) ListProjectJobArtifacts(ctx context.Context, projectID int64, jobID int64) ([]cidomain.ProjectJobArtifact, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return nil, err
	}
	items, err := s.artifactRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetProjectJobArtifactContent(ctx context.Context, projectID int64, jobID int64, artifactID int64) (ProjectJobArtifactContent, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return ProjectJobArtifactContent{}, err
	}
	artifact, err := s.artifactRepo.GetByProjectJobAndID(ctx, projectID, jobID, artifactID)
	if err != nil {
		if err == storageports.ErrNotFound {
			return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact not found", err)
		}
		return ProjectJobArtifactContent{}, err
	}
	content, err := s.storage.Load(ctx, artifact.StorageKey)
	if err != nil {
		return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact content not found", err)
	}
	return ProjectJobArtifactContent{Artifact: artifact, ContentBase64: base64.StdEncoding.EncodeToString(content)}, nil
}

func (s *Service) UploadProjectJobArtifact(ctx context.Context, projectID int64, jobID int64, input UploadArtifactInput) (cidomain.ProjectJobArtifact, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, apperror.NotFound("project not found", err)
	}
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	fileName := strings.TrimSpace(input.FileName)
	if fileName == "" {
		return cidomain.ProjectJobArtifact{}, apperror.BadRequest("artifact file_name is required", fmt.Errorf("artifact file_name is required"))
	}
	if strings.TrimSpace(input.ContentBase64) == "" {
		return cidomain.ProjectJobArtifact{}, apperror.BadRequest("artifact content_base64 is required", fmt.Errorf("artifact content_base64 is required"))
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return cidomain.ProjectJobArtifact{}, fmt.Errorf("decode artifact content: %w", err)
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
		return cidomain.ProjectJobArtifact{}, err
	}
	storageKey, err := s.storage.SavePipelineArtifact(ctx, project.FullPath, jobID, artifact.ID, fileName, content, contentType)
	if err != nil {
		_ = s.artifactRepo.DeleteByID(ctx, artifact.ID)
		return cidomain.ProjectJobArtifact{}, err
	}
	if err := s.artifactRepo.MarkStored(ctx, artifact.ID, storageports.StoreProjectJobArtifactInput{ContentType: contentType, ByteSize: int64(len(content)), StorageKey: storageKey}); err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	return s.artifactRepo.GetByID(ctx, artifact.ID)
}

func (s *Service) AppendProjectJobTrace(ctx context.Context, projectID int64, jobID int64, input AppendTraceInput) (ProjectJobTrace, error) {
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
		return ProjectJobTrace{}, err
	}
	return s.GetProjectJobTrace(ctx, projectID, jobID)
}

func (s *Service) RunNext(ctx context.Context, workerID string, lease time.Duration) (bool, error) {
	job, claimed, err := s.jobRepo.ClaimNextByKinds(ctx, []string{KindNoop}, normalizeWorkerID(workerID), lease)
	if err != nil || !claimed {
		return claimed, err
	}
	result, execErr := s.execute(ctx, job)
	if execErr != nil {
		if err := s.jobRepo.MarkFailed(ctx, job, execErr.Error(), retryDelay(job.Attempts)); err != nil {
			return true, fmt.Errorf("record project job failure after execution error %q: %w", execErr.Error(), err)
		}
		s.logger.Warn("project job failed", slog.Int64("job_id", job.ID), slog.String("kind", job.Kind), slog.String("error", execErr.Error()))
		return true, nil
	}
	if err := s.jobRepo.MarkSucceeded(ctx, job.ID, result); err != nil {
		return true, err
	}
	s.logger.Info("project job completed", slog.Int64("job_id", job.ID), slog.String("kind", job.Kind))
	return true, nil
}

func (s *Service) ClaimProjectJob(ctx context.Context, projectID int64, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return cidomain.ProjectJob{}, false, apperror.NotFound("project not found", err)
	}
	return s.jobRepo.ClaimNextByProjectIDAndKinds(ctx, projectID, []string{KindScript}, normalizeWorkerID(workerID), lease)
}

func (s *Service) CompleteProjectJob(ctx context.Context, projectID int64, jobID int64, result string) (cidomain.ProjectJob, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if item.Status != storageports.ProjectJobStatusRunning {
		return cidomain.ProjectJob{}, apperror.Conflict("project job is not running", fmt.Errorf("project job is not running: %s", item.Status))
	}
	if err := s.jobRepo.MarkSucceeded(ctx, item.ID, result); err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := s.recordScriptLog(ctx, item, result, ""); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return s.GetProjectJob(ctx, projectID, jobID)
}

func (s *Service) FailProjectJob(ctx context.Context, projectID int64, jobID int64, message string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
	return s.FailProjectJobWithResult(ctx, projectID, jobID, message, "", retryAfter)
}

func (s *Service) FailProjectJobWithResult(ctx context.Context, projectID int64, jobID int64, message string, result string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
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
		return cidomain.ProjectJob{}, err
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
		return "", fmt.Errorf("unsupported project job kind: %s", item.Kind)
	}
}

func (s *Service) recordScriptLog(ctx context.Context, item cidomain.ProjectJob, result string, fallback string) error {
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
	return err
}

func parseScriptResult(result string, fallback string) scriptResult {
	var parsed scriptResult
	trimmed := strings.TrimSpace(result)
	if trimmed != "" {
		if err := json.Unmarshal([]byte(trimmed), &parsed); err == nil {
			return parsed
		}
		if index := strings.LastIndex(trimmed, "\n{"); index >= 0 {
			if err := json.Unmarshal([]byte(strings.TrimSpace(trimmed[index+1:])), &parsed); err == nil {
				if prefix := strings.TrimSpace(trimmed[:index]); prefix != "" && parsed.Output == "" {
					parsed.Output = prefix
				}
				return parsed
			}
		}
		parsed.Output = trimmed
		return parsed
	}
	parsed.Output = strings.TrimSpace(fallback)
	if parsed.Output != "" {
		parsed.ExitCode = 1
	}
	return parsed
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
