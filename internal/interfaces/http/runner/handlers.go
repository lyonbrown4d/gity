package runner

import (
	"context"
	"time"

	runnerservice "github.com/lyonbrown4d/gity/internal/application/runner"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
)

func (e *Endpoint) listProjectRunners(ctx context.Context, in *projectRunnersInput) (*runnerOutput, error) {
	items, err := e.service.ListProjectRunners(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: toRunnerViews(items)}, nil
}

func (e *Endpoint) registerProjectRunner(ctx context.Context, in *registerRunnerInput) (*runnerOutput, error) {
	input, err := mapperx.MapStrict[runnerservice.RegisterInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.RegisterProjectRunner(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: toRegistrationView(item)}, nil
}

func (e *Endpoint) deleteProjectRunner(ctx context.Context, in *projectRunnerInput) (*runnerOutput, error) {
	item, err := e.service.DeleteProjectRunner(ctx, in.ProjectID, in.RunnerID)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: toRunnerView(item)}, nil
}

func (e *Endpoint) listProjectVariables(ctx context.Context, in *projectVariablesInput) (*runnerOutput, error) {
	items, err := e.service.ListProjectVariables(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: toVariableViews(items)}, nil
}

func (e *Endpoint) upsertProjectVariable(ctx context.Context, in *upsertVariableInput) (*runnerOutput, error) {
	input, err := mapperx.MapStrict[runnerservice.VariableInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.UpsertProjectVariable(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: toVariableView(item)}, nil
}

func (e *Endpoint) deleteProjectVariable(ctx context.Context, in *projectVariableInput) (*runnerOutput, error) {
	if err := e.service.DeleteProjectVariable(ctx, in.ProjectID, in.Key); err != nil {
		return nil, err
	}
	return &runnerOutput{Body: map[string]any{"deleted": true}}, nil
}

func (e *Endpoint) heartbeat(ctx context.Context, in *runnerTokenInput) (*runnerOutput, error) {
	item, err := e.service.Heartbeat(ctx, in.Body.Token)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) claimJob(ctx context.Context, in *claimJobInput) (*runnerOutput, error) {
	item, err := e.service.ClaimJob(ctx, in.Body.Token, time.Duration(in.Body.LeaseSeconds)*time.Second)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) completeJob(ctx context.Context, in *runnerCompleteJobInput) (*runnerOutput, error) {
	item, err := e.service.CompleteJob(ctx, in.Body.Token, in.JobID, in.Body.Result)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) failJob(ctx context.Context, in *runnerFailJobInput) (*runnerOutput, error) {
	item, err := e.service.FailJob(ctx, in.Body.Token, in.JobID, in.Body.Error, in.Body.Result, runnerRetryDelay(in.Body.RetryAfterSeconds))
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) appendTrace(ctx context.Context, in *runnerTraceInput) (*runnerOutput, error) {
	input, err := mapperx.MapStrict[runnerservice.AppendTraceInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.AppendTrace(ctx, in.Body.Token, in.JobID, input)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) downloadSourceArchive(ctx context.Context, in *runnerSourceArchiveInput) (*runnerOutput, error) {
	item, err := e.service.DownloadSourceArchive(ctx, in.Body.Token, in.JobID)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func (e *Endpoint) uploadArtifact(ctx context.Context, in *runnerArtifactInput) (*runnerOutput, error) {
	input, err := mapperx.MapStrict[runnerservice.UploadArtifactInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	item, err := e.service.UploadArtifact(ctx, in.Body.Token, in.JobID, input)
	if err != nil {
		return nil, err
	}
	return &runnerOutput{Body: item}, nil
}

func runnerRetryDelay(seconds int) time.Duration {
	return time.Duration(seconds) * time.Second
}
