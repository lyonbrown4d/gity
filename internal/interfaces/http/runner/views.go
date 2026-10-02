package runner

import (
	"strconv"
	"time"

	streamx "github.com/arcgolabs/collectionx/stream"
	runnerservice "github.com/lyonbrown4d/gity/internal/application/runner"
)

type runnerView struct {
	ID            string `json:"id"`
	ProjectID     string `json:"project_id"`
	Name          string `json:"name"`
	Description   string `json:"description,omitempty"`
	Tags          string `json:"tags,omitempty"`
	Status        string `json:"status"`
	Active        bool   `json:"active"`
	LastContactAt string `json:"last_contact_at,omitempty"`
	CreatedAt     string `json:"created_at,omitempty"`
	UpdatedAt     string `json:"updated_at,omitempty"`
}

type registrationView struct {
	Runner runnerView `json:"runner"`
	Token  string     `json:"token"`
}

type variableView struct {
	ID        string `json:"id"`
	ProjectID string `json:"project_id"`
	Key       string `json:"key"`
	Value     string `json:"value,omitempty"`
	Masked    bool   `json:"masked"`
	Protected bool   `json:"protected"`
	CreatedAt string `json:"created_at,omitempty"`
	UpdatedAt string `json:"updated_at,omitempty"`
}

func toRunnerViews(items []runnerservice.RunnerView) []runnerView {
	return collectViews(streamx.Of(items...).Map[runnerView](toRunnerView))
}

func toRunnerView(item runnerservice.RunnerView) runnerView {
	view := runnerView{
		ID:          formatRunnerID(item.ID),
		ProjectID:   formatRunnerID(item.ProjectID),
		Name:        item.Name,
		Description: item.Description,
		Tags:        item.Tags,
		Status:      item.Status,
		Active:      item.Active,
		CreatedAt:   formatRunnerTime(item.CreatedAt),
		UpdatedAt:   formatRunnerTime(item.UpdatedAt),
	}
	if item.LastContactAt != nil {
		view.LastContactAt = formatRunnerTime(*item.LastContactAt)
	}
	return view
}

func toRegistrationView(item runnerservice.RegistrationView) registrationView {
	return registrationView{
		Runner: toRunnerView(item.Runner),
		Token:  item.Token,
	}
}

func toVariableViews(items []runnerservice.VariableView) []variableView {
	return collectViews(streamx.Of(items...).Map[variableView](toVariableView))
}

func toVariableView(item runnerservice.VariableView) variableView {
	return variableView{
		ID:        formatRunnerID(item.ID),
		ProjectID: formatRunnerID(item.ProjectID),
		Key:       item.Key,
		Value:     item.Value,
		Masked:    item.Masked,
		Protected: item.Protected,
		CreatedAt: formatRunnerTime(item.CreatedAt),
		UpdatedAt: formatRunnerTime(item.UpdatedAt),
	}
}

func formatRunnerID(value int64) string {
	return strconv.FormatInt(value, 10)
}

func formatRunnerTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}

func collectViews[T any](items streamx.Stream[T]) []T {
	views := items.ToSlice()
	if views == nil {
		return []T{}
	}
	return views
}
