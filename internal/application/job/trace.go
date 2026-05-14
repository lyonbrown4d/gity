package job

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

type AppendTraceInput struct {
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
}

type TracePageInput struct {
	Offset int `json:"offset"`
	Limit  int `json:"limit"`
}

type ProjectJobTrace struct {
	Job             cidomain.ProjectJob      `json:"job"`
	Logs            []cidomain.ProjectJobLog `json:"logs"`
	Trace           string                   `json:"trace"`
	TraceOffset     int                      `json:"trace_offset"`
	TraceLimit      int                      `json:"trace_limit"`
	TraceTotal      int                      `json:"trace_total"`
	HasMore         bool                     `json:"has_more"`
	ExitCode        int                      `json:"exit_code"`
	OutputTruncated bool                     `json:"output_truncated"`
	DurationMillis  int64                    `json:"duration_millis"`
}

type scriptResult struct {
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	WorkDir         string `json:"work_dir"`
}

func (s *Service) GetProjectJobTrace(ctx context.Context, projectID, jobID int64) (ProjectJobTrace, error) {
	return s.GetProjectJobTracePage(ctx, projectID, jobID, TracePageInput{})
}

func (s *Service) GetProjectJobTracePage(ctx context.Context, projectID, jobID int64, input TracePageInput) (ProjectJobTrace, error) {
	job, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
	}
	logs, err := s.logRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "list project job trace")
	}
	trace := ProjectJobTrace{Job: job, Logs: logs.Values()}
	if logs.Len() == 0 {
		trace.Trace = fallbackTrace(job)
		applyTracePage(&trace, input)
		return trace, nil
	}
	latest, _ := logs.Get(logs.Len() - 1)
	trace.Trace = latest.Output
	trace.ExitCode = latest.ExitCode
	trace.OutputTruncated = latest.OutputTruncated == 1
	trace.DurationMillis = latest.DurationMillis
	applyTracePage(&trace, input)
	return trace, nil
}

func applyTracePage(trace *ProjectJobTrace, input TracePageInput) {
	total := len(trace.Trace)
	offset := min(max(input.Offset, 0), total)
	limit := max(input.Limit, 0)
	end := total
	if limit > 0 && offset+limit < end {
		end = offset + limit
	}
	trace.TraceTotal = total
	trace.TraceOffset = offset
	trace.TraceLimit = limit
	trace.HasMore = end < total
	trace.Trace = trace.Trace[offset:end]
}

func (s *Service) AppendProjectJobTrace(ctx context.Context, projectID, jobID int64, input AppendTraceInput) (ProjectJobTrace, error) {
	item, err := s.GetProjectJob(ctx, projectID, jobID)
	if err != nil {
		return ProjectJobTrace{}, err
	}
	if item.Kind != KindScript {
		return ProjectJobTrace{}, apperror.BadRequest("project job does not support trace streaming", fmt.Errorf("project job kind: %s", item.Kind))
	}
	if item.Status != storageports.ProjectJobStatusRunning {
		return ProjectJobTrace{}, apperror.Conflict("project job is not running", fmt.Errorf("project job status: %s", item.Status))
	}
	if s.logRepo == nil {
		return ProjectJobTrace{}, apperror.Internal("project job log repository is not configured")
	}
	if input.Output == "" && !input.OutputTruncated {
		return s.GetProjectJobTrace(ctx, projectID, jobID)
	}
	if _, err := s.logRepo.Append(ctx, storageports.AppendProjectJobLogInput{
		ProjectID:       projectID,
		ProjectJobID:    jobID,
		Attempt:         item.Attempts,
		Output:          input.Output,
		OutputTruncated: input.OutputTruncated,
		DurationMillis:  input.DurationMillis,
	}); err != nil {
		return ProjectJobTrace{}, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "append project job trace")
	}
	return s.GetProjectJobTrace(ctx, projectID, jobID)
}

func (s *Service) recordScriptLog(ctx context.Context, item cidomain.ProjectJob, result, fallback string) error {
	if item.Kind != KindScript || s.logRepo == nil {
		return nil
	}
	parsed := parseScriptResult(result, fallback)
	_, err := s.logRepo.UpsertAttempt(ctx, storageports.CreateProjectJobLogInput{
		ProjectID:       item.ProjectID,
		ProjectJobID:    item.ID,
		Attempt:         item.Attempts,
		ExitCode:        parsed.ExitCode,
		Output:          parsed.Output,
		OutputTruncated: parsed.OutputTruncated,
		DurationMillis:  parsed.DurationMillis,
	})
	if err != nil {
		return oops.In("job").With("project_id", item.ProjectID, "job_id", item.ID, "attempt", item.Attempts).Wrapf(err, "record project job script log")
	}
	return nil
}

func parseScriptResult(result, fallback string) scriptResult {
	trimmed := strings.TrimSpace(result)
	if trimmed == "" {
		return fallbackScriptResult(fallback)
	}
	if parsed, ok := decodeScriptResult(trimmed); ok {
		return parsed
	}
	return scriptResult{Output: trimmed}
}

func fallbackScriptResult(fallback string) scriptResult {
	parsed := scriptResult{Output: strings.TrimSpace(fallback)}
	if parsed.Output != "" {
		parsed.ExitCode = 1
	}
	return parsed
}

func decodeScriptResult(trimmed string) (scriptResult, bool) {
	var parsed scriptResult
	if err := json.Unmarshal([]byte(trimmed), &parsed); err == nil {
		return parsed, true
	}
	index := strings.LastIndex(trimmed, "\n{")
	if index < 0 {
		return scriptResult{}, false
	}
	if err := json.Unmarshal([]byte(strings.TrimSpace(trimmed[index+1:])), &parsed); err != nil {
		return scriptResult{}, false
	}
	if prefix := strings.TrimSpace(trimmed[:index]); prefix != "" && parsed.Output == "" {
		parsed.Output = prefix
	}
	return parsed, true
}

func fallbackTrace(job cidomain.ProjectJob) string {
	if strings.TrimSpace(job.Result) != "" {
		return parseScriptResult(job.Result, "").Output
	}
	return strings.TrimSpace(job.LastError)
}
