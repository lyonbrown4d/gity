package pipeline

import (
	"time"

	"github.com/lyonbrown4d/gity/internal/ci/plan_dsl"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

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
	Tags        []string                    `json:"tags,omitempty"`
}

type scriptJobPayload struct {
	PipelineID      int64             `json:"pipeline_id"`
	PipelineIID     int64             `json:"pipeline_iid"`
	PipelineName    string            `json:"pipeline_name"`
	ProjectFullPath string            `json:"project_full_path"`
	RefName         string            `json:"ref_name"`
	CommitSHA       string            `json:"commit_sha"`
	Stage           string            `json:"stage"`
	Image           string            `json:"image,omitempty"`
	Needs           []string          `json:"needs,omitempty"`
	Script          []string          `json:"script"`
	Artifacts       []string          `json:"artifacts,omitempty"`
	Tags            []string          `json:"tags,omitempty"`
	Env             map[string]string `json:"env,omitempty"`
	MaskedValues    []string          `json:"masked_values,omitempty"`
	TimeoutSeconds  int               `json:"timeout_seconds"`
	ConfigSource    string            `json:"config_source"`
	PipelineJobName string            `json:"pipeline_job_name"`
}

const (
	defaultCIConfigPath = ".gity-ci.plano"
	pipelineSourcePush  = "push"
)

var blockedRunAfter = time.Date(9999, time.January, 1, 0, 0, 0, 0, time.UTC)
