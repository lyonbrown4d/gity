package runneragent

import (
	"encoding/json"
	"strconv"
	"strings"
	"time"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

type projectJobWire struct {
	ID          wireInt64 `json:"id"`
	ProjectID   wireInt64 `json:"project_id"`
	Kind        string    `json:"kind"`
	Status      string    `json:"status"`
	Payload     string    `json:"payload"`
	Result      string    `json:"result"`
	Attempts    int       `json:"attempts"`
	MaxAttempts int       `json:"max_attempts"`
	RunAfter    time.Time `json:"run_after"`
	LockedBy    string    `json:"locked_by"`
	LockedUntil time.Time `json:"locked_until"`
	LastError   string    `json:"last_error"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
	StartedAt   time.Time `json:"started_at"`
	FinishedAt  time.Time `json:"finished_at"`
}

func (item projectJobWire) ProjectJob() cidomain.ProjectJob {
	return cidomain.ProjectJob{
		ID:          int64(item.ID),
		ProjectID:   int64(item.ProjectID),
		Kind:        item.Kind,
		Status:      item.Status,
		Payload:     item.Payload,
		Result:      item.Result,
		Attempts:    item.Attempts,
		MaxAttempts: item.MaxAttempts,
		RunAfter:    item.RunAfter,
		LockedBy:    item.LockedBy,
		LockedUntil: item.LockedUntil,
		LastError:   item.LastError,
		CreatedAt:   item.CreatedAt,
		UpdatedAt:   item.UpdatedAt,
		StartedAt:   item.StartedAt,
		FinishedAt:  item.FinishedAt,
	}
}

type wireInt64 int64

func (id *wireInt64) UnmarshalJSON(content []byte) error {
	raw := strings.TrimSpace(string(content))
	if raw == "" || raw == "null" {
		*id = 0
		return nil
	}
	if strings.HasPrefix(raw, `"`) {
		var text string
		if err := json.Unmarshal(content, &text); err != nil {
			return oops.In("runner_agent").Wrapf(err, "decode response id")
		}
		return id.parse(text)
	}
	return id.parse(raw)
}

func (id *wireInt64) parse(value string) error {
	normalized := strings.TrimSpace(value)
	if normalized == "" {
		*id = 0
		return nil
	}
	parsed, err := strconv.ParseInt(normalized, 10, 64)
	if err != nil {
		return oops.In("runner_agent").With("value", normalized).Wrapf(err, "parse response id")
	}
	*id = wireInt64(parsed)
	return nil
}
