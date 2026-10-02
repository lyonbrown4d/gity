package pipeline

import (
	"context"

	"github.com/arcgolabs/mapper"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	"github.com/samber/oops"
)

func (e *Endpoint) listPipelines(ctx context.Context, in *projectPipelinesInput) (*pipelineOutput, error) {
	items, err := e.service.ListPipelines(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toProjectPipelineViews(items)}, nil
}

func (e *Endpoint) createPipeline(ctx context.Context, in *createPipelineInput) (*pipelineOutput, error) {
	input, err := e.mapper.Map[pipelineservice.CreatePipelineInput](in.Body, mapper.Strict())
	if err != nil {
		return nil, oops.In("mapper").Wrapf(err, "map value")
	}
	item, err := e.service.CreatePipeline(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineDetailView(item)}, nil
}

func (e *Endpoint) lintPipeline(ctx context.Context, in *lintPipelineInput) (*pipelineOutput, error) {
	input, err := e.mapper.Map[pipelineservice.LintInput](in.Body, mapper.Strict())
	if err != nil {
		return nil, oops.In("mapper").Wrapf(err, "map value")
	}
	item, err := e.service.LintPipeline(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: item}, nil
}

func (e *Endpoint) getPipeline(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
	item, err := e.service.GetPipeline(ctx, in.ProjectID, in.PipelineID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineDetailView(item)}, nil
}

func (e *Endpoint) refreshPipeline(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
	item, err := e.service.RefreshPipeline(ctx, in.ProjectID, in.PipelineID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineDetailView(item)}, nil
}

func (e *Endpoint) cancelPipeline(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
	item, err := e.service.CancelPipeline(ctx, in.ProjectID, in.PipelineID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineDetailView(item)}, nil
}

func (e *Endpoint) retryPipeline(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
	item, err := e.service.RetryPipeline(ctx, in.ProjectID, in.PipelineID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineDetailView(item)}, nil
}

func (e *Endpoint) listPipelineJobs(ctx context.Context, in *projectPipelineInput) (*pipelineOutput, error) {
	items, err := e.service.ListPipelineJobs(ctx, in.ProjectID, in.PipelineID)
	if err != nil {
		return nil, err
	}
	return &pipelineOutput{Body: toPipelineJobViews(items)}, nil
}
