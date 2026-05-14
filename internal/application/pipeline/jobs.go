package pipeline

import (
	"context"
	"errors"
	"fmt"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/ci/plan_dsl"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

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
	if err := s.retryPipelineJobs(ctx, projectID, pipelineID, items.Values()); err != nil {
		return PipelineView{}, err
	}
	if err := s.pipelineRepo.UpdateStatus(ctx, pipeline, gitports.ProjectPipelineStatusPending); err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "status", gitports.ProjectPipelineStatusPending).Wrapf(err, "update pipeline status")
	}
	return s.RefreshPipeline(ctx, projectID, pipelineID)
}

func (s *Service) retryPipelineJobs(ctx context.Context, projectID, pipelineID int64, items []cidomain.ProjectPipelineJob) error {
	for i := range items {
		if err := s.retryPipelineJob(ctx, projectID, pipelineID, items[i]); err != nil {
			return err
		}
	}
	return nil
}

func (s *Service) retryPipelineJob(ctx context.Context, projectID, pipelineID int64, item cidomain.ProjectPipelineJob) error {
	job, err := s.jobRepo.GetByID(ctx, item.ProjectJobID)
	if err != nil {
		return oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", item.ProjectJobID).Wrapf(err, "load pipeline project job")
	}
	if job.Status == gitports.ProjectJobStatusRunning {
		return apperror.Conflict("pipeline has running job", fmt.Errorf("project job is running: %s", job.Status))
	}
	needs, err := decodeStringSlice(item.Needs)
	if err != nil {
		return err
	}
	if err := s.jobRepo.RetryByID(ctx, job.ID, retryRunAfter(needs)); err != nil {
		return oops.In("pipeline").With("project_id", projectID, "pipeline_id", pipelineID, "project_job_id", job.ID).Wrapf(err, "retry pipeline job")
	}
	return nil
}

func retryRunAfter(needs []string) time.Time {
	if len(needs) == 0 {
		return time.Now().UTC()
	}
	return blockedRunAfter
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

func (s *Service) enqueueStage(ctx context.Context, project projectdomain.Project, pipeline cidomain.ProjectPipeline, stage plandsl.StageSpec, index int, runAfter time.Time) (PipelineJobView, error) {
	env, maskedValues, err := s.pipelineVariables(ctx, project, pipeline)
	if err != nil {
		return PipelineJobView{}, err
	}
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
		Tags:            stage.Tags,
		Env:             env,
		MaskedValues:    maskedValues,
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
		Tags:        stage.Tags,
	}, nil
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
	tags, err := decodeJobTags(job.Payload)
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
		Tags:        tags,
	}, nil
}

func (s *Service) pipelineVariables(ctx context.Context, project projectdomain.Project, pipeline cidomain.ProjectPipeline) (map[string]string, []string, error) {
	if s.variableRepo == nil {
		return nil, nil, nil
	}
	items, err := s.variableRepo.ListByProjectID(ctx, project.ID)
	if err != nil {
		return nil, nil, oops.In("pipeline").With("project_id", project.ID, "pipeline_id", pipeline.ID).Wrapf(err, "list project ci variables")
	}
	env := make(map[string]string, items.Len())
	maskedValues := make([]string, 0, items.Len())
	for _, item := range items.Values() {
		if item.IsProtected() && pipeline.RefName != project.DefaultBranch {
			continue
		}
		env[item.Key] = item.Value
		if item.IsMasked() && item.Value != "" {
			maskedValues = append(maskedValues, item.Value)
		}
	}
	return env, maskedValues, nil
}
