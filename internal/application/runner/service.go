package runner

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectrunner"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
	"net/http"
	"strings"
	"time"
)

type Service struct {
	projectRepo *projectrepo.Repository
	runnerRepo  *projectrunnerrepo.Repository
	jobService  *jobservice.Service
	pipelineSvc *pipelineservice.Service
	gitRunner   *gitexec.Runner
}

type RegisterInput struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Tags        string `json:"tags"`
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

type SourceArchiveView struct {
	FileName      string `json:"file_name"`
	Encoding      string `json:"encoding"`
	ContentBase64 string `json:"content_base64"`
}

type scriptSourcePayload struct {
	ProjectFullPath string `json:"project_full_path"`
	RefName         string `json:"ref_name"`
	CommitSHA       string `json:"commit_sha"`
}

type RunnerView struct {
	ID            int64      `json:"id"`
	ProjectID     int64      `json:"project_id"`
	Name          string     `json:"name"`
	Description   string     `json:"description"`
	Tags          string     `json:"tags"`
	Status        string     `json:"status"`
	Active        bool       `json:"active"`
	LastContactAt *time.Time `json:"last_contact_at,omitempty"`
	CreatedAt     time.Time  `json:"created_at"`
	UpdatedAt     time.Time  `json:"updated_at"`
}

type RegistrationView struct {
	Runner RunnerView `json:"runner"`
	Token  string     `json:"token"`
}

type ClaimView struct {
	Claimed bool                `json:"claimed"`
	Runner  RunnerView          `json:"runner"`
	Job     cidomain.ProjectJob `json:"job,omitempty"`
}

func NewService(projectRepo *projectrepo.Repository, runnerRepo *projectrunnerrepo.Repository, jobService *jobservice.Service, pipelineSvc *pipelineservice.Service, gitRunner *gitexec.Runner) *Service {
	return &Service{projectRepo: projectRepo, runnerRepo: runnerRepo, jobService: jobService, pipelineSvc: pipelineSvc, gitRunner: gitRunner}
}

func (s *Service) ListProjectRunners(ctx context.Context, projectID int64) ([]RunnerView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.runnerRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	views := make([]RunnerView, 0, items.Len())
	for _, item := range items.Values() {
		views = append(views, toRunnerView(item))
	}
	return views, nil
}

func (s *Service) RegisterProjectRunner(ctx context.Context, projectID int64, input RegisterInput) (RegistrationView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return RegistrationView{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		return RegistrationView{}, httpx.NewError(http.StatusBadRequest, "runner name is required", fmt.Errorf("runner name is required"))
	}
	token, err := generateRunnerToken()
	if err != nil {
		return RegistrationView{}, err
	}
	item, err := s.runnerRepo.Create(ctx, projectrunnerrepo.CreateInput{
		ProjectID:   projectID,
		Name:        name,
		Description: input.Description,
		Tags:        input.Tags,
		Token:       token,
	})
	if err != nil {
		return RegistrationView{}, err
	}
	return RegistrationView{Runner: toRunnerView(item), Token: token}, nil
}

func (s *Service) DeleteProjectRunner(ctx context.Context, projectID int64, runnerID int64) (RunnerView, error) {
	item, err := s.runnerRepo.GetByProjectAndID(ctx, projectID, runnerID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return RunnerView{}, httpx.NewError(http.StatusNotFound, "project runner not found", err)
		}
		return RunnerView{}, err
	}
	if err := s.runnerRepo.DeleteByID(ctx, item.ID); err != nil {
		return RunnerView{}, err
	}
	return toRunnerView(item), nil
}

func (s *Service) Heartbeat(ctx context.Context, token string) (RunnerView, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return RunnerView{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return RunnerView{}, err
	}
	runner, err = s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		return RunnerView{}, err
	}
	return toRunnerView(runner), nil
}

func (s *Service) ClaimJob(ctx context.Context, token string, lease time.Duration) (ClaimView, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return ClaimView{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return ClaimView{}, err
	}
	job, claimed, err := s.jobService.ClaimProjectJob(ctx, runner.ProjectID, runnerWorkerID(runner), lease)
	if err != nil {
		return ClaimView{}, err
	}
	if claimed {
		if err := s.refreshPipelineForJob(ctx, runner.ProjectID, job.ID); err != nil {
			return ClaimView{}, err
		}
	}
	runner, err = s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		return ClaimView{}, err
	}
	return ClaimView{Claimed: claimed, Runner: toRunnerView(runner), Job: job}, nil
}

func (s *Service) CompleteJob(ctx context.Context, token string, jobID int64, result string) (cidomain.ProjectJob, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return cidomain.ProjectJob{}, err
	}
	completed, err := s.jobService.CompleteProjectJob(ctx, runner.ProjectID, jobID, result)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := s.refreshPipelineForJob(ctx, runner.ProjectID, jobID); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return completed, nil
}

func (s *Service) FailJob(ctx context.Context, token string, jobID int64, message string, result string, retryAfter time.Duration) (cidomain.ProjectJob, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return cidomain.ProjectJob{}, err
	}
	failed, err := s.jobService.FailProjectJobWithResult(ctx, runner.ProjectID, jobID, message, result, retryAfter)
	if err != nil {
		return cidomain.ProjectJob{}, err
	}
	if err := s.refreshPipelineForJob(ctx, runner.ProjectID, jobID); err != nil {
		return cidomain.ProjectJob{}, err
	}
	return failed, nil
}

func (s *Service) UploadArtifact(ctx context.Context, token string, jobID int64, input UploadArtifactInput) (cidomain.ProjectJobArtifact, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	return s.jobService.UploadProjectJobArtifact(ctx, runner.ProjectID, jobID, jobservice.UploadArtifactInput{
		Name:          input.Name,
		FileName:      input.FileName,
		FilePath:      input.FilePath,
		ContentType:   input.ContentType,
		ContentBase64: input.ContentBase64,
	})
}

func (s *Service) AppendTrace(ctx context.Context, token string, jobID int64, input AppendTraceInput) (jobservice.ProjectJobTrace, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return jobservice.ProjectJobTrace{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return jobservice.ProjectJobTrace{}, err
	}
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return jobservice.ProjectJobTrace{}, err
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return jobservice.ProjectJobTrace{}, err
	}
	return s.jobService.AppendProjectJobTrace(ctx, runner.ProjectID, jobID, jobservice.AppendTraceInput{
		Output:          input.Output,
		OutputTruncated: input.OutputTruncated,
		DurationMillis:  input.DurationMillis,
	})
}

func (s *Service) DownloadSourceArchive(ctx context.Context, token string, jobID int64) (SourceArchiveView, error) {
	runner, err := s.authenticateRunner(ctx, token)
	if err != nil {
		return SourceArchiveView{}, err
	}
	job, err := s.jobService.GetProjectJob(ctx, runner.ProjectID, jobID)
	if err != nil {
		return SourceArchiveView{}, err
	}
	if err := ensureRunnerOwnsJob(runner, job); err != nil {
		return SourceArchiveView{}, err
	}
	if s.gitRunner == nil {
		return SourceArchiveView{}, httpx.NewError(http.StatusInternalServerError, "git runner is not configured")
	}
	project, err := s.projectRepo.GetByID(ctx, runner.ProjectID)
	if err != nil {
		return SourceArchiveView{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	payload, err := decodeScriptSourcePayload(job)
	if err != nil {
		return SourceArchiveView{}, err
	}
	if payload.ProjectFullPath != "" && payload.ProjectFullPath != project.FullPath {
		return SourceArchiveView{}, httpx.NewError(http.StatusBadRequest, "job project path does not match runner project", fmt.Errorf("payload project=%q expected=%q", payload.ProjectFullPath, project.FullPath))
	}
	revision := firstNonEmpty(payload.CommitSHA, payload.RefName)
	if revision == "" {
		return SourceArchiveView{}, httpx.NewError(http.StatusBadRequest, "job source revision is required", fmt.Errorf("job source revision is required"))
	}
	if err := s.runnerRepo.MarkHeartbeat(ctx, runner.ID); err != nil {
		return SourceArchiveView{}, err
	}
	content, err := s.gitRunner.Archive(ctx, project.FullPath+".git", revision)
	if err != nil {
		return SourceArchiveView{}, err
	}
	return SourceArchiveView{
		FileName:      fmt.Sprintf("project-%d-job-%d-source.zip", project.ID, job.ID),
		Encoding:      "base64",
		ContentBase64: base64.StdEncoding.EncodeToString(content),
	}, nil
}

func (s *Service) authenticateRunner(ctx context.Context, token string) (cidomain.ProjectRunner, error) {
	token = strings.TrimSpace(token)
	if token == "" {
		return cidomain.ProjectRunner{}, httpx.NewError(http.StatusUnauthorized, "runner token is required", fmt.Errorf("runner token is required"))
	}
	runner, err := s.runnerRepo.GetByToken(ctx, token)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return cidomain.ProjectRunner{}, httpx.NewError(http.StatusUnauthorized, "invalid runner token", err)
		}
		return cidomain.ProjectRunner{}, err
	}
	if runner.Active != 1 {
		return cidomain.ProjectRunner{}, httpx.NewError(http.StatusForbidden, "runner is disabled", fmt.Errorf("runner is disabled"))
	}
	return runner, nil
}

func decodeScriptSourcePayload(job cidomain.ProjectJob) (scriptSourcePayload, error) {
	if strings.TrimSpace(job.Kind) != jobservice.KindScript {
		return scriptSourcePayload{}, httpx.NewError(http.StatusBadRequest, "job source archive is only available for script jobs", fmt.Errorf("job kind: %s", job.Kind))
	}
	var payload scriptSourcePayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return scriptSourcePayload{}, httpx.NewError(http.StatusBadRequest, "invalid script job payload", err)
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

func ensureRunnerOwnsJob(runner cidomain.ProjectRunner, job cidomain.ProjectJob) error {
	expected := runnerWorkerID(runner)
	if strings.TrimSpace(job.LockedBy) != expected {
		return httpx.NewError(http.StatusConflict, "project job is not claimed by runner", fmt.Errorf("project job locked_by=%q expected=%q", job.LockedBy, expected))
	}
	return nil
}

func (s *Service) refreshPipelineForJob(ctx context.Context, projectID int64, jobID int64) error {
	if s.pipelineSvc == nil {
		return nil
	}
	return s.pipelineSvc.RefreshProjectJob(ctx, projectID, jobID)
}

func toRunnerView(item cidomain.ProjectRunner) RunnerView {
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
		Status:        item.Status,
		Active:        item.Active == 1,
		LastContactAt: lastContactAt,
		CreatedAt:     item.CreatedAt,
		UpdatedAt:     item.UpdatedAt,
	}
}

func runnerWorkerID(item cidomain.ProjectRunner) string {
	return fmt.Sprintf("runner:%d", item.ID)
}

func generateRunnerToken() (string, error) {
	var buffer [32]byte
	if _, err := rand.Read(buffer[:]); err != nil {
		return "", fmt.Errorf("generate runner token: %w", err)
	}
	return "grt_" + base64.RawURLEncoding.EncodeToString(buffer[:]), nil
}
