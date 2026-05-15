package runneragent

import (
	"context"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type ScriptPayload struct {
	ProjectFullPath string            `json:"project_full_path"`
	RefName         string            `json:"ref_name"`
	CommitSHA       string            `json:"commit_sha"`
	Script          []string          `json:"script"`
	ExecutionMode   string            `json:"execution_mode,omitempty"`
	Image           string            `json:"image,omitempty"`
	Env             map[string]string `json:"env"`
	Shell           string            `json:"shell"`
	Artifacts       []string          `json:"artifacts"`
	Tags            []string          `json:"tags"`
	MaskedValues    []string          `json:"masked_values"`
	TimeoutSeconds  int               `json:"timeout_seconds"`
}

type ScriptResult struct {
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	WorkDir         string `json:"work_dir"`
}

type ScriptCancellationChecker func(ctx context.Context) (bool, error)

type ScriptTraceStreamer func(ctx context.Context, output string, outputTruncated bool, durationMillis int64) error

type ScriptSourceFetcher func(ctx context.Context, job cidomain.ProjectJob, payload ScriptPayload, workDir string) error
