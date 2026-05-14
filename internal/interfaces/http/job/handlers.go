package job

import (
	"context"

	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
)

func (e *Endpoint) listProjectJobs(ctx context.Context, in *projectJobsInput) (*jobOutput, error) {
	items, err := e.service.ListProjectJobs(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: items}, nil
}

func (e *Endpoint) createJob(ctx context.Context, in *createJobInput) (*jobOutput, error) {
	input, err := mapperx.MapStrict[jobservice.CreateInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.EnqueueProjectJob(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}

func (e *Endpoint) getProjectJob(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
	item, err := e.service.GetProjectJob(ctx, in.ProjectID, in.JobID)
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}

func (e *Endpoint) cancelProjectJob(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
	item, err := e.service.CancelProjectJob(ctx, in.ProjectID, in.JobID)
	if err != nil {
		return nil, err
	}
	if err := e.refreshPipelineForJob(ctx, in.ProjectID, in.JobID); err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}

func (e *Endpoint) retryProjectJob(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
	item, err := e.service.RetryProjectJob(ctx, in.ProjectID, in.JobID)
	if err != nil {
		return nil, err
	}
	if err := e.refreshPipelineForJob(ctx, in.ProjectID, in.JobID); err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}

func (e *Endpoint) getProjectJobTrace(ctx context.Context, in *projectJobTraceInput) (*jobOutput, error) {
	item, err := e.service.GetProjectJobTracePage(ctx, in.ProjectID, in.JobID, jobservice.TracePageInput{Offset: in.Offset, Limit: in.Limit})
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}

func (e *Endpoint) listProjectJobArtifacts(ctx context.Context, in *projectJobInput) (*jobOutput, error) {
	items, err := e.service.ListProjectJobArtifacts(ctx, in.ProjectID, in.JobID)
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: items}, nil
}

func (e *Endpoint) getProjectJobArtifact(ctx context.Context, in *projectJobArtifactInput) (*jobOutput, error) {
	item, err := e.service.GetProjectJobArtifactContent(ctx, in.ProjectID, in.JobID, in.ArtifactID)
	if err != nil {
		return nil, err
	}
	return &jobOutput{Body: item}, nil
}
