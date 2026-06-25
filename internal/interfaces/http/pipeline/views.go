package pipeline

import (
	"strconv"
	"time"

	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	plandsl "github.com/lyonbrown4d/gity/internal/ci/plan_dsl"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type projectPipelineView struct {
	ID            string `json:"id"`
	ProjectID     string `json:"project_id"`
	IID           int64  `json:"iid"`
	Name          string `json:"name"`
	Source        string `json:"source"`
	RefName       string `json:"ref_name"`
	CommitSHA     string `json:"commit_sha"`
	Status        string `json:"status"`
	ConfigSource  string `json:"config_source"`
	ConfigContent string `json:"config_content,omitempty"`
	CreatedAt     string `json:"created_at,omitempty"`
	UpdatedAt     string `json:"updated_at,omitempty"`
	StartedAt     string `json:"started_at,omitempty"`
	FinishedAt    string `json:"finished_at,omitempty"`
}

type pipelineDetailView struct {
	Pipeline projectPipelineView  `json:"pipeline"`
	Spec     plandsl.PipelineSpec `json:"spec"`
	Jobs     []pipelineJobView    `json:"jobs,omitempty"`
}

type pipelineJobView struct {
	PipelineJob projectPipelineJobLinkView `json:"pipeline_job"`
	ProjectJob  pipelineProjectJobView     `json:"project_job"`
	Status      string                     `json:"status"`
	Needs       []string                   `json:"needs"`
	Script      []string                   `json:"script"`
	Artifacts   []string                   `json:"artifacts,omitempty"`
	Tags        []string                   `json:"tags,omitempty"`
}

type projectPipelineJobLinkView struct {
	ID           string `json:"id"`
	ProjectID    string `json:"project_id"`
	PipelineID   string `json:"pipeline_id"`
	ProjectJobID string `json:"project_job_id"`
	Name         string `json:"name"`
	Stage        string `json:"stage"`
	Needs        string `json:"needs,omitempty"`
	Image        string `json:"image,omitempty"`
	Script       string `json:"script,omitempty"`
	Artifacts    string `json:"artifacts,omitempty"`
	SortOrder    int    `json:"sort_order"`
	CreatedAt    string `json:"created_at,omitempty"`
	UpdatedAt    string `json:"updated_at,omitempty"`
}

type pipelineProjectJobView struct {
	ID          string `json:"id"`
	ProjectID   string `json:"project_id"`
	Kind        string `json:"kind"`
	Status      string `json:"status"`
	Payload     string `json:"payload,omitempty"`
	Result      string `json:"result,omitempty"`
	Attempts    int    `json:"attempts"`
	MaxAttempts int    `json:"max_attempts"`
	RunAfter    string `json:"run_after,omitempty"`
	LockedBy    string `json:"locked_by,omitempty"`
	LockedUntil string `json:"locked_until,omitempty"`
	LastError   string `json:"last_error,omitempty"`
	CreatedAt   string `json:"created_at,omitempty"`
	UpdatedAt   string `json:"updated_at,omitempty"`
	StartedAt   string `json:"started_at,omitempty"`
	FinishedAt  string `json:"finished_at,omitempty"`
}

func toProjectPipelineViews(items []cidomain.ProjectPipeline) []projectPipelineView {
	views := make([]projectPipelineView, 0, len(items))
	for index := range items {
		views = append(views, toProjectPipelineView(items[index]))
	}
	return views
}

func toProjectPipelineView(item cidomain.ProjectPipeline) projectPipelineView {
	return projectPipelineView{
		ID:            formatPipelineID(item.ID),
		ProjectID:     formatPipelineID(item.ProjectID),
		IID:           item.IID,
		Name:          item.Name,
		Source:        item.Source,
		RefName:       item.RefName,
		CommitSHA:     item.CommitSHA,
		Status:        item.Status,
		ConfigSource:  item.ConfigSource,
		ConfigContent: item.ConfigContent,
		CreatedAt:     formatPipelineTime(item.CreatedAt),
		UpdatedAt:     formatPipelineTime(item.UpdatedAt),
		StartedAt:     formatPipelineTime(item.StartedAt),
		FinishedAt:    formatPipelineTime(item.FinishedAt),
	}
}

func toPipelineDetailView(item pipelineservice.PipelineView) pipelineDetailView {
	return pipelineDetailView{
		Pipeline: toProjectPipelineView(item.Pipeline),
		Spec:     item.Spec,
		Jobs:     toPipelineJobViews(item.Jobs),
	}
}

func toPipelineJobViews(items []pipelineservice.PipelineJobView) []pipelineJobView {
	views := make([]pipelineJobView, 0, len(items))
	for index := range items {
		views = append(views, toPipelineJobView(items[index]))
	}
	return views
}

func toPipelineJobView(item pipelineservice.PipelineJobView) pipelineJobView {
	return pipelineJobView{
		PipelineJob: toProjectPipelineJobLinkView(item.PipelineJob),
		ProjectJob:  toPipelineProjectJobView(item.ProjectJob),
		Status:      item.Status,
		Needs:       item.Needs,
		Script:      item.Script,
		Artifacts:   item.Artifacts,
		Tags:        item.Tags,
	}
}

func toProjectPipelineJobLinkView(item cidomain.ProjectPipelineJob) projectPipelineJobLinkView {
	return projectPipelineJobLinkView{
		ID:           formatPipelineID(item.ID),
		ProjectID:    formatPipelineID(item.ProjectID),
		PipelineID:   formatPipelineID(item.PipelineID),
		ProjectJobID: formatPipelineID(item.ProjectJobID),
		Name:         item.Name,
		Stage:        item.Stage,
		Needs:        item.Needs,
		Image:        item.Image,
		Script:       item.Script,
		Artifacts:    item.Artifacts,
		SortOrder:    item.SortOrder,
		CreatedAt:    formatPipelineTime(item.CreatedAt),
		UpdatedAt:    formatPipelineTime(item.UpdatedAt),
	}
}

func toPipelineProjectJobView(item cidomain.ProjectJob) pipelineProjectJobView {
	return pipelineProjectJobView{
		ID:          formatPipelineID(item.ID),
		ProjectID:   formatPipelineID(item.ProjectID),
		Kind:        item.Kind,
		Status:      item.Status,
		Payload:     item.Payload,
		Result:      item.Result,
		Attempts:    item.Attempts,
		MaxAttempts: item.MaxAttempts,
		RunAfter:    formatPipelineTime(item.RunAfter),
		LockedBy:    item.LockedBy,
		LockedUntil: formatPipelineTime(item.LockedUntil),
		LastError:   item.LastError,
		CreatedAt:   formatPipelineTime(item.CreatedAt),
		UpdatedAt:   formatPipelineTime(item.UpdatedAt),
		StartedAt:   formatPipelineTime(item.StartedAt),
		FinishedAt:  formatPipelineTime(item.FinishedAt),
	}
}

func formatPipelineID(value int64) string {
	return strconv.FormatInt(value, 10)
}

func formatPipelineTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}
