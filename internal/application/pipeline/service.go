package pipeline

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"strings"
	"time"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/ci/plan_dsl"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/samber/oops"
)

type Service struct {
	projectRepo     gitports.ProjectRepository
	pipelineRepo    gitports.ProjectPipelineRepository
	pipelineJobRepo gitports.ProjectPipelineJobRepository
	jobService      *jobservice.Service
	jobRepo         gitports.ProjectJobRepository
	gitRepo         gitports.GitRepository
}

type CreatePipelineInput struct {
	Source        string `json:"source"`
	RefName       string `json:"ref_name"`
	CommitSHA     string `json:"commit_sha"`
	ConfigSource  string `json:"config_source"`
	ConfigContent string `json:"config_content"`
}

type LintInput struct {
	ConfigContent string `json:"config_content"`
}

type PipelineView struct {
	Pipeline cidomain.ProjectPipeline `json:"pipeline"`
	Spec     plandsl.PipelineSpec     `json:"spec,omitzero"`
	Jobs     []PipelineJobView        `json:"jobs,omitempty"`
}

type PipelineJobView struct {
	PipelineJob cidomain.ProjectPipelineJob `json:"pipeline_job"`
	ProjectJob  cidomain.ProjectJob         `json:"project_job"`
	Status      string                      `json:"status"`
	Needs       []string                    `json:"needs"`
	Script      []string                    `json:"script"`
	Artifacts   []string                    `json:"artifacts,omitempty"`
}

type scriptJobPayload struct {
	PipelineID      int64    `json:"pipeline_id"`
	PipelineIID     int64    `json:"pipeline_iid"`
	PipelineName    string   `json:"pipeline_name"`
	ProjectFullPath string   `json:"project_full_path"`
	RefName         string   `json:"ref_name"`
	CommitSHA       string   `json:"commit_sha"`
	Stage           string   `json:"stage"`
	Image           string   `json:"image,omitempty"`
	Needs           []string `json:"needs,omitempty"`
	Script          []string `json:"script"`
	Artifacts       []string `json:"artifacts,omitempty"`
	TimeoutSeconds  int      `json:"timeout_seconds"`
	ConfigSource    string   `json:"config_source"`
	PipelineJobName string   `json:"pipeline_job_name"`
}

const (
	defaultCIConfigPath = ".gity-ci.plano"
	pipelineSourcePush  = "push"
)

var blockedRunAfter = time.Date(9999, time.January, 1, 0, 0, 0, 0, time.UTC)

func NewService(
	projectRepo gitports.ProjectRepository,
	pipelineRepo gitports.ProjectPipelineRepository,
	pipelineJobRepo gitports.ProjectPipelineJobRepository,
	jobService *jobservice.Service,
	jobRepo gitports.ProjectJobRepository,
	gitRepo gitports.GitRepository,
) *Service {
	return &Service{
		projectRepo:     projectRepo,
		pipelineRepo:    pipelineRepo,
		pipelineJobRepo: pipelineJobRepo,
		jobService:      jobService,
		jobRepo:         jobRepo,
		gitRepo:         gitRepo,
	}
}

func (s *Service) ListPipelines(ctx context.Context, projectID int64) ([]cidomain.ProjectPipeline, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.pipelineRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("pipeline").With("project_id", projectID).Wrapf(err, "list project pipelines")
	}
	return items.Values(), nil
}

func (s *Service) GetPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	item, err := s.refreshPipelineState(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	jobs, err := s.listPipelineJobs(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	return PipelineView{Pipeline: item, Jobs: jobs}, nil
}

func (s *Service) LintPipeline(ctx context.Context, projectID int64, input LintInput) (plandsl.PipelineSpec, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return plandsl.PipelineSpec{}, apperror.NotFound("project not found", err)
	}
	return s.compileConfig(ctx, input.ConfigContent)
}

func (s *Service) CreatePipeline(ctx context.Context, projectID int64, input CreatePipelineInput) (PipelineView, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return PipelineView{}, apperror.NotFound("project not found", err)
	}
	spec, err := s.compileConfig(ctx, input.ConfigContent)
	if err != nil {
		return PipelineView{}, err
	}
	pipeline, err := s.pipelineRepo.Create(ctx, gitports.CreateProjectPipelineInput{
		ProjectID:     projectID,
		Name:          spec.Name,
		Source:        input.Source,
		RefName:       input.RefName,
		CommitSHA:     input.CommitSHA,
		Status:        gitports.ProjectPipelineStatusPending,
		ConfigSource:  input.ConfigSource,
		ConfigContent: input.ConfigContent,
	})
	if err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "source", input.Source, "ref", input.RefName, "commit_sha", input.CommitSHA).Wrapf(err, "create pipeline")
	}
	jobs, err := collectionlist.ReduceErrList(collectionlist.NewList(spec.Stages...), make([]PipelineJobView, 0, len(spec.Stages)), func(acc []PipelineJobView, index int, stage plandsl.StageSpec) ([]PipelineJobView, error) {
		view, enqueueErr := s.enqueueStage(ctx, project, pipeline, stage, index, initialRunAfter(stage))
		if enqueueErr != nil {
			return acc, enqueueErr
		}
		return append(acc, view), nil
	})
	if err != nil {
		return PipelineView{}, fmt.Errorf("enqueue pipeline stages: %w", err)
	}
	return PipelineView{Pipeline: pipeline, Spec: spec, Jobs: jobs}, nil
}

func (s *Service) CreatePushPipeline(ctx context.Context, projectID int64, refName, commitSHA string) (PipelineView, bool, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return PipelineView{}, false, apperror.NotFound("project not found", err)
	}
	configContent, ok, err := s.loadRepositoryConfig(ctx, project, refName, commitSHA)
	if err != nil || !ok {
		return PipelineView{}, false, err
	}
	existing, err := s.pipelineRepo.GetByProjectSourceRefCommit(ctx, projectID, pipelineSourcePush, refName, commitSHA)
	if err == nil {
		view, viewErr := s.GetPipeline(ctx, projectID, existing.ID)
		return view, false, viewErr
	}
	if !errors.Is(err, gitports.ErrNotFound) {
		return PipelineView{}, false, oops.In("pipeline").With("project_id", projectID, "source", pipelineSourcePush, "ref", refName, "commit_sha", commitSHA).Wrapf(err, "load existing push pipeline")
	}
	view, err := s.CreatePipeline(ctx, projectID, CreatePipelineInput{
		Source:        pipelineSourcePush,
		RefName:       refName,
		CommitSHA:     commitSHA,
		ConfigSource:  defaultCIConfigPath,
		ConfigContent: configContent,
	})
	if err != nil {
		return PipelineView{}, false, err
	}
	return view, true, nil
}

func (s *Service) ListPipelineJobs(ctx context.Context, projectID, pipelineID int64) ([]PipelineJobView, error) {
	if _, err := s.refreshPipelineState(ctx, projectID, pipelineID); err != nil {
		return nil, err
	}
	return s.listPipelineJobs(ctx, projectID, pipelineID)
}

func (s *Service) RefreshPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	pipeline, err := s.refreshPipelineState(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	jobs, err := s.listPipelineJobs(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	return PipelineView{Pipeline: pipeline, Jobs: jobs}, nil
}

func (s *Service) RetryPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	pipeline, err := s.loadPipeline(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	if !isTerminalPipelineStatus(pipeline.Status) && pipeline.Status != gitports.ProjectPipelineStatusPending {
		return PipelineView{}, apperror.Conflict("pipeline is not retryable", fmt.Errorf("pipeline status: %s", pipeline.Status))
	}
	items, err := s.pipelineJobRepo.ListByPipelineID(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "list pipeline jobs")
	}
	itemValues := items.Values()
	for i := range itemValues {
		item := itemValues[i]
		job, err := s.jobRepo.GetByID(ctx, item.ProjectJobID)
		if err != nil {
			return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", item.ProjectJobID).Wrapf(err, "load pipeline project job")
		}
		if job.Status == gitports.ProjectJobStatusRunning {
			return PipelineView{}, apperror.Conflict("pipeline has running job", fmt.Errorf("project job is running: %s", job.Status))
		}
		needs, err := decodeStringSlice(item.Needs)
		if err != nil {
			return PipelineView{}, err
		}
		runAfter := blockedRunAfter
		if len(needs) == 0 {
			runAfter = time.Now().UTC()
		}
		if err := s.jobRepo.RetryByID(ctx, job.ID, runAfter); err != nil {
			return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", job.ID).Wrapf(err, "retry pipeline job")
		}
	}
	if err := s.pipelineRepo.UpdateStatus(ctx, pipeline, gitports.ProjectPipelineStatusPending); err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "status", gitports.ProjectPipelineStatusPending).Wrapf(err, "update pipeline status")
	}
	return s.RefreshPipeline(ctx, projectID, pipelineID)
}

func (s *Service) CancelPipeline(ctx context.Context, projectID, pipelineID int64) (PipelineView, error) {
	pipeline, err := s.loadPipeline(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, err
	}
	items, err := s.pipelineJobRepo.ListByPipelineID(ctx, projectID, pipelineID)
	if err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "list pipeline jobs")
	}
	itemValues := items.Values()
	for i := range itemValues {
		item := itemValues[i]
		job, err := s.jobRepo.GetByID(ctx, item.ProjectJobID)
		if err != nil {
			return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", item.ProjectJobID).Wrapf(err, "load pipeline project job")
		}
		if isTerminalJobStatus(job.Status) {
			continue
		}
		if err := s.jobRepo.CancelByID(ctx, job.ID); err != nil {
			return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", job.ID).Wrapf(err, "cancel pipeline job")
		}
	}
	if err := s.pipelineRepo.UpdateStatus(ctx, pipeline, gitports.ProjectPipelineStatusCancelled); err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "status", gitports.ProjectPipelineStatusCancelled).Wrapf(err, "update pipeline status")
	}
	return s.GetPipeline(ctx, projectID, pipelineID)
}

func (s *Service) RefreshProjectJob(ctx context.Context, projectID, projectJobID int64) error {
	item, err := s.pipelineJobRepo.GetByProjectJobID(ctx, projectID, projectJobID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return nil
		}
		return oops.In("pipeline").With("project_id", projectID, "project_job_id", projectJobID).Wrapf(err, "load pipeline job by project job")
	}
	_, err = s.refreshPipelineState(ctx, item.ProjectID, item.PipelineID)
	return err
}

func (s *Service) listPipelineJobs(ctx context.Context, projectID, pipelineID int64) ([]PipelineJobView, error) {
	items, err := s.pipelineJobRepo.ListByPipelineID(ctx, projectID, pipelineID)
	if err != nil {
		return nil, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID).Wrapf(err, "list pipeline jobs")
	}
	views, err := collectionlist.ReduceErrList(items, make([]PipelineJobView, 0, items.Len()), func(acc []PipelineJobView, _ int, item cidomain.ProjectPipelineJob) ([]PipelineJobView, error) {
		view, viewErr := s.toPipelineJobView(ctx, item)
		if viewErr != nil {
			return acc, viewErr
		}
		return append(acc, view), nil
	})
	if err != nil {
		return nil, fmt.Errorf("build pipeline job views: %w", err)
	}
	return views, nil
}

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

func (s *Service) compileConfig(ctx context.Context, content string) (plandsl.PipelineSpec, error) {
	if strings.TrimSpace(content) == "" {
		return plandsl.PipelineSpec{}, apperror.BadRequest("ci config content is required", errors.New("ci config content is required"))
	}
	spec, err := plandsl.Compile(ctx, ".gity-ci.plano", content)
	if err != nil {
		return plandsl.PipelineSpec{}, apperror.BadRequest("invalid ci plano config", err)
	}
	return spec, nil
}

func (s *Service) enqueueStage(ctx context.Context, project projectdomain.Project, pipeline cidomain.ProjectPipeline, stage plandsl.StageSpec, index int, runAfter time.Time) (PipelineJobView, error) {
	payload, err := encodeScriptPayload(scriptJobPayload{
		PipelineID:      pipeline.ID,
		PipelineIID:     pipeline.IID,
		PipelineName:    pipeline.Name,
		ProjectFullPath: project.FullPath,
		RefName:         pipeline.RefName,
		CommitSHA:       pipeline.CommitSHA,
		Stage:           stage.Name,
		Image:           stage.Image,
		Needs:           stage.Needs,
		Script:          stage.Script,
		Artifacts:       stage.Artifacts,
		TimeoutSeconds:  stage.TimeoutSeconds,
		ConfigSource:    pipeline.ConfigSource,
		PipelineJobName: stage.Name,
	})
	if err != nil {
		return PipelineJobView{}, oops.In("pipeline").With("project_id", pipeline.ProjectID, "pipeline_id", pipeline.ID, "stage", stage.Name).Wrapf(err, "encode script job payload")
	}
	projectJob, err := s.jobService.EnqueueProjectJob(ctx, pipeline.ProjectID, jobservice.CreateInput{
		Kind:        jobservice.KindScript,
		Payload:     payload,
		MaxAttempts: 1,
		RunAfter:    runAfter,
	})
	if err != nil {
		return PipelineJobView{}, oops.In("pipeline").With("project_id", pipeline.ProjectID, "pipeline_id", pipeline.ID, "stage", stage.Name).Wrapf(err, "enqueue project job")
	}
	needs, err := encodeStringSlice(stage.Needs)
	if err != nil {
		return PipelineJobView{}, err
	}
	script, err := encodeStringSlice(stage.Script)
	if err != nil {
		return PipelineJobView{}, err
	}
	artifacts, err := encodeStringSlice(stage.Artifacts)
	if err != nil {
		return PipelineJobView{}, err
	}
	pipelineJob, err := s.pipelineJobRepo.Create(ctx, gitports.CreateProjectPipelineJobInput{
		ProjectID:    pipeline.ProjectID,
		PipelineID:   pipeline.ID,
		ProjectJobID: projectJob.ID,
		Name:         stage.Name,
		Stage:        stage.Name,
		Needs:        needs,
		Image:        stage.Image,
		Script:       script,
		Artifacts:    artifacts,
		SortOrder:    index,
	})
	if err != nil {
		return PipelineJobView{}, oops.In("pipeline").With("project_id", pipeline.ProjectID, "pipeline_id", pipeline.ID, "project_job_id", projectJob.ID, "stage", stage.Name).Wrapf(err, "create pipeline job")
	}
	return PipelineJobView{
		PipelineJob: pipelineJob,
		ProjectJob:  projectJob,
		Status:      pipelineJobStatus(projectJob, stage.Needs),
		Needs:       stage.Needs,
		Script:      stage.Script,
		Artifacts:   stage.Artifacts,
	}, nil
}

func (s *Service) loadRepositoryConfig(ctx context.Context, project projectdomain.Project, refName, commitSHA string) (string, bool, error) {
	if s.gitRepo == nil {
		return "", false, apperror.Internal("git repository service is not configured")
	}
	revision := strings.TrimSpace(commitSHA)
	if revision == "" {
		revision = strings.TrimSpace(refName)
	}
	blob, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", revision, project.DefaultBranch, defaultCIConfigPath)
	if err != nil {
		if errors.Is(err, gitports.ErrPathNotFound) || errors.Is(err, gitports.ErrReferenceNotFound) || errors.Is(err, gitports.ErrEmptyRepository) {
			return "", false, nil
		}
		return "", false, oops.In("pipeline").With("project_id", project.ID, "ref", refName, "commit_sha", commitSHA, "path", defaultCIConfigPath).Wrapf(err, "load repository ci config")
	}
	if blob.Encoding != "utf-8" {
		return "", false, apperror.BadRequest("ci config must be utf-8 text", fmt.Errorf("ci config encoding: %s", blob.Encoding))
	}
	return blob.Content, true, nil
}

func (s *Service) toPipelineJobView(ctx context.Context, item cidomain.ProjectPipelineJob) (PipelineJobView, error) {
	job, err := s.jobRepo.GetByID(ctx, item.ProjectJobID)
	if err != nil {
		return PipelineJobView{}, oops.In("pipeline").With("project_id", item.ProjectID, "pipeline_id", item.PipelineID, "project_job_id", item.ProjectJobID).Wrapf(err, "load pipeline project job")
	}
	needs, err := decodeStringSlice(item.Needs)
	if err != nil {
		return PipelineJobView{}, err
	}
	script, err := decodeStringSlice(item.Script)
	if err != nil {
		return PipelineJobView{}, err
	}
	artifacts, err := decodeStringSlice(item.Artifacts)
	if err != nil {
		return PipelineJobView{}, err
	}
	return PipelineJobView{
		PipelineJob: item,
		ProjectJob:  job,
		Status:      pipelineJobStatus(job, needs),
		Needs:       needs,
		Script:      script,
		Artifacts:   artifacts,
	}, nil
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

func encodeScriptPayload(payload scriptJobPayload) (string, error) {
	raw, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("encode script job payload: %w", err)
	}
	return string(raw), nil
}

func encodeStringSlice(values []string) (string, error) {
	if values == nil {
		values = []string{}
	}
	raw, err := json.Marshal(values)
	if err != nil {
		return "", fmt.Errorf("encode string slice: %w", err)
	}
	return string(raw), nil
}

func decodeStringSlice(value string) ([]string, error) {
	if strings.TrimSpace(value) == "" {
		return nil, nil
	}
	var out []string
	if err := json.Unmarshal([]byte(value), &out); err != nil {
		return nil, fmt.Errorf("decode string slice: %w", err)
	}
	return out, nil
}

func initialRunAfter(stage plandsl.StageSpec) time.Time {
	if len(stage.Needs) == 0 {
		return time.Time{}
	}
	return blockedRunAfter
}

func isBlockedRunAfter(value time.Time) bool {
	return value.UTC().Year() >= blockedRunAfter.Year()
}

func pipelineJobStatus(job cidomain.ProjectJob, needs []string) string {
	if job.Status == gitports.ProjectJobStatusPending && len(needs) > 0 && isBlockedRunAfter(job.RunAfter) {
		return "blocked"
	}
	return job.Status
}

func isTerminalPipelineStatus(status string) bool {
	switch status {
	case gitports.ProjectPipelineStatusSucceeded, gitports.ProjectPipelineStatusFailed, gitports.ProjectPipelineStatusCancelled:
		return true
	default:
		return false
	}
}

func isTerminalJobStatus(status string) bool {
	switch status {
	case gitports.ProjectJobStatusSucceeded, gitports.ProjectJobStatusFailed, gitports.ProjectJobStatusCancelled:
		return true
	default:
		return false
	}
}

func pipelineStatus(jobs map[string]cidomain.ProjectJob) string {
	if len(jobs) == 0 {
		return gitports.ProjectPipelineStatusPending
	}
	allSucceeded := true
	anyRunning := false
	anyStarted := false
	anyCancelled := false
	for name := range jobs {
		job := jobs[name]
		switch job.Status {
		case gitports.ProjectJobStatusFailed:
			return gitports.ProjectPipelineStatusFailed
		case gitports.ProjectJobStatusCancelled:
			anyCancelled = true
			allSucceeded = false
		case gitports.ProjectJobStatusSucceeded:
			anyStarted = true
		case gitports.ProjectJobStatusRunning:
			anyRunning = true
			anyStarted = true
			allSucceeded = false
		default:
			allSucceeded = false
		}
	}
	if allSucceeded {
		return gitports.ProjectPipelineStatusSucceeded
	}
	if anyCancelled {
		return gitports.ProjectPipelineStatusCancelled
	}
	if anyRunning || anyStarted {
		return gitports.ProjectPipelineStatusRunning
	}
	return gitports.ProjectPipelineStatusPending
}

func needsSucceeded(needs []string, jobs map[string]cidomain.ProjectJob) bool {
	for _, need := range needs {
		job, ok := jobs[need]
		if !ok || job.Status != gitports.ProjectJobStatusSucceeded {
			return false
		}
	}
	return true
}
