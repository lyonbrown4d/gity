package pipeline

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/ci/plan_dsl"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

func encodeScriptPayload(payload scriptJobPayload) (string, error) {
	raw, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("encode script job payload: %w", err)
	}
	return string(raw), nil
}

func encodeStringSlice(values []string) (string, error) {
	if values == nil {
		values = []string{}
	}
	raw, err := json.Marshal(values)
	if err != nil {
		return "", fmt.Errorf("encode string slice: %w", err)
	}
	return string(raw), nil
}

func decodeStringSlice(value string) ([]string, error) {
	if strings.TrimSpace(value) == "" {
		return nil, nil
	}
	var out []string
	if err := json.Unmarshal([]byte(value), &out); err != nil {
		return nil, fmt.Errorf("decode string slice: %w", err)
	}
	return out, nil
}

func initialRunAfter(stage plandsl.StageSpec) time.Time {
	if len(stage.Needs) == 0 {
		return time.Time{}
	}
	return blockedRunAfter
}

func isBlockedRunAfter(value time.Time) bool {
	return value.UTC().Year() >= blockedRunAfter.Year()
}

func pipelineJobStatus(job cidomain.ProjectJob, needs []string) string {
	if job.Status == gitports.ProjectJobStatusPending && len(needs) > 0 && isBlockedRunAfter(job.RunAfter) {
		return "blocked"
	}
	return job.Status
}

func isTerminalPipelineStatus(status string) bool {
	switch status {
	case gitports.ProjectPipelineStatusSucceeded, gitports.ProjectPipelineStatusFailed, gitports.ProjectPipelineStatusCancelled:
		return true
	default:
		return false
	}
}

func isTerminalJobStatus(status string) bool {
	switch status {
	case gitports.ProjectJobStatusSucceeded, gitports.ProjectJobStatusFailed, gitports.ProjectJobStatusCancelled:
		return true
	default:
		return false
	}
}

func pipelineStatus(jobs map[string]cidomain.ProjectJob) string {
	if len(jobs) == 0 {
		return gitports.ProjectPipelineStatusPending
	}
	summary := pipelineJobStatusSummary{allSucceeded: true}
	for name := range jobs {
		if summary.observe(jobs[name]) {
			return gitports.ProjectPipelineStatusFailed
		}
	}
	return summary.status()
}

type pipelineJobStatusSummary struct {
	allSucceeded bool
	anyRunning   bool
	anyStarted   bool
	anyCancelled bool
}

func (s *pipelineJobStatusSummary) observe(job cidomain.ProjectJob) bool {
	switch job.Status {
	case gitports.ProjectJobStatusFailed:
		return true
	case gitports.ProjectJobStatusCancelled:
		s.anyCancelled = true
		s.allSucceeded = false
	case gitports.ProjectJobStatusSucceeded:
		s.anyStarted = true
	case gitports.ProjectJobStatusRunning:
		s.anyRunning = true
		s.anyStarted = true
		s.allSucceeded = false
	default:
		s.allSucceeded = false
	}
	return false
}

func (s pipelineJobStatusSummary) status() string {
	if s.allSucceeded {
		return gitports.ProjectPipelineStatusSucceeded
	}
	if s.anyCancelled {
		return gitports.ProjectPipelineStatusCancelled
	}
	if s.anyRunning || s.anyStarted {
		return gitports.ProjectPipelineStatusRunning
	}
	return gitports.ProjectPipelineStatusPending
}

func needsSucceeded(needs []string, jobs map[string]cidomain.ProjectJob) bool {
	for _, need := range needs {
		job, ok := jobs[need]
		if !ok || job.Status != gitports.ProjectJobStatusSucceeded {
			return false
		}
	}
	return true
}
