package job

import (
	"strconv"
	"time"

	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type projectJobView struct {
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

type projectJobTraceView struct {
	Job             projectJobView      `json:"job"`
	Logs            []projectJobLogView `json:"logs,omitempty"`
	Trace           string              `json:"trace"`
	TraceOffset     int                 `json:"trace_offset"`
	TraceLimit      int                 `json:"trace_limit"`
	TraceTotal      int                 `json:"trace_total"`
	HasMore         bool                `json:"has_more"`
	ExitCode        int                 `json:"exit_code"`
	OutputTruncated bool                `json:"output_truncated"`
	DurationMillis  int64               `json:"duration_millis"`
}

type projectJobLogView struct {
	ID              string `json:"id"`
	ProjectID       string `json:"project_id"`
	ProjectJobID    string `json:"project_job_id"`
	Attempt         int    `json:"attempt"`
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	CreatedAt       string `json:"created_at,omitempty"`
	UpdatedAt       string `json:"updated_at,omitempty"`
}

type projectJobArtifactView struct {
	ID           string `json:"id"`
	ProjectID    string `json:"project_id"`
	ProjectJobID string `json:"project_job_id"`
	Name         string `json:"name"`
	FileName     string `json:"file_name"`
	FilePath     string `json:"file_path,omitempty"`
	ContentType  string `json:"content_type,omitempty"`
	ByteSize     int64  `json:"byte_size"`
	CreatedAt    string `json:"created_at,omitempty"`
	UpdatedAt    string `json:"updated_at,omitempty"`
}

type projectJobArtifactContentView struct {
	Artifact      projectJobArtifactView `json:"artifact"`
	ContentBase64 string                 `json:"content_base64"`
}

func toProjectJobViews(items []cidomain.ProjectJob) []projectJobView {
	views := make([]projectJobView, 0, len(items))
	for index := range items {
		views = append(views, toProjectJobView(items[index]))
	}
	return views
}

func toProjectJobView(item cidomain.ProjectJob) projectJobView {
	return projectJobView{
		ID:          formatID(item.ID),
		ProjectID:   formatID(item.ProjectID),
		Kind:        item.Kind,
		Status:      item.Status,
		Payload:     item.Payload,
		Result:      item.Result,
		Attempts:    item.Attempts,
		MaxAttempts: item.MaxAttempts,
		RunAfter:    formatTime(item.RunAfter),
		LockedBy:    item.LockedBy,
		LockedUntil: formatTime(item.LockedUntil),
		LastError:   item.LastError,
		CreatedAt:   formatTime(item.CreatedAt),
		UpdatedAt:   formatTime(item.UpdatedAt),
		StartedAt:   formatTime(item.StartedAt),
		FinishedAt:  formatTime(item.FinishedAt),
	}
}

func toProjectJobTraceView(item jobservice.ProjectJobTrace) projectJobTraceView {
	return projectJobTraceView{
		Job:             toProjectJobView(item.Job),
		Logs:            toProjectJobLogViews(item.Logs),
		Trace:           item.Trace,
		TraceOffset:     item.TraceOffset,
		TraceLimit:      item.TraceLimit,
		TraceTotal:      item.TraceTotal,
		HasMore:         item.HasMore,
		ExitCode:        item.ExitCode,
		OutputTruncated: item.OutputTruncated,
		DurationMillis:  item.DurationMillis,
	}
}

func toProjectJobLogViews(items []cidomain.ProjectJobLog) []projectJobLogView {
	views := make([]projectJobLogView, 0, len(items))
	for index := range items {
		item := items[index]
		views = append(views, projectJobLogView{
			ID:              formatID(item.ID),
			ProjectID:       formatID(item.ProjectID),
			ProjectJobID:    formatID(item.ProjectJobID),
			Attempt:         item.Attempt,
			ExitCode:        item.ExitCode,
			Output:          item.Output,
			OutputTruncated: item.OutputTruncated == 1,
			DurationMillis:  item.DurationMillis,
			CreatedAt:       formatTime(item.CreatedAt),
			UpdatedAt:       formatTime(item.UpdatedAt),
		})
	}
	return views
}

func toProjectJobArtifactViews(items []cidomain.ProjectJobArtifact) []projectJobArtifactView {
	views := make([]projectJobArtifactView, 0, len(items))
	for index := range items {
		views = append(views, toProjectJobArtifactView(items[index]))
	}
	return views
}

func toProjectJobArtifactView(item cidomain.ProjectJobArtifact) projectJobArtifactView {
	return projectJobArtifactView{
		ID:           formatID(item.ID),
		ProjectID:    formatID(item.ProjectID),
		ProjectJobID: formatID(item.ProjectJobID),
		Name:         item.Name,
		FileName:     item.FileName,
		FilePath:     item.FilePath,
		ContentType:  item.ContentType,
		ByteSize:     item.ByteSize,
		CreatedAt:    formatTime(item.CreatedAt),
		UpdatedAt:    formatTime(item.UpdatedAt),
	}
}

func toProjectJobArtifactContentView(item jobservice.ProjectJobArtifactContent) projectJobArtifactContentView {
	return projectJobArtifactContentView{
		Artifact:      toProjectJobArtifactView(item.Artifact),
		ContentBase64: item.ContentBase64,
	}
}

func formatID(value int64) string {
	return strconv.FormatInt(value, 10)
}

func formatTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}
