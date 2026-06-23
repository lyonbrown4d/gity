package runneragent

import (
	"context"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type RunnerClient interface {
	Heartbeat(ctx context.Context) error
	ClaimJob(ctx context.Context, leaseSeconds int) (ClaimResponse, error)
	GetProjectJob(ctx context.Context, projectID, jobID int64) (cidomain.ProjectJob, error)
	CompleteJob(ctx context.Context, jobID int64, result string) error
	FailJob(ctx context.Context, jobID int64, message, result string, retryAfterSeconds int) error
	AppendTrace(ctx context.Context, jobID int64, output string, outputTruncated bool, durationMillis int64) error
	DownloadSourceArchive(ctx context.Context, jobID int64) ([]byte, error)
	UploadArtifact(ctx context.Context, jobID int64, artifact ArtifactFile) error
}

type ScriptExecutor interface {
	ExecuteScriptJob(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer, sourceFetcher ScriptSourceFetcher) (string, error)
}

type ScriptExecutorFunc func(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer, sourceFetcher ScriptSourceFetcher) (string, error)

func (f ScriptExecutorFunc) ExecuteScriptJob(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer, sourceFetcher ScriptSourceFetcher) (string, error) {
	return f(ctx, cfg, job, checker, traceStreamer, sourceFetcher)
}

type AgentDependencies struct {
	Client         RunnerClient
	ScriptExecutor ScriptExecutor
}

func NewClientFromConfig(cfg Config) RunnerClient {
	return NewClient(cfg.ServerURL, cfg.Token)
}

func NewAgentDependencies(cfg Config) AgentDependencies {
	return AgentDependencies{
		Client:         NewClientFromConfig(cfg),
		ScriptExecutor: ScriptExecutorFunc(ExecuteScriptJobWithSource),
	}
}
