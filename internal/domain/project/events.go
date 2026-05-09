// Package project defines project domain models and events.
package project

import domainevent "github.com/DaiYuANg/gity/internal/domain/event"

const (
	EventProjectCreated = "project.created"
	EventProjectDeleted = "project.deleted"
)

type ProjectCreated struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	OrganizationID int64  `json:"organization_id"`
	ProjectName    string `json:"name"`
	PathKey        string `json:"path_key"`
	FullPath       string `json:"full_path"`
	Visibility     string `json:"visibility"`
	DefaultBranch  string `json:"default_branch"`
}

func (ProjectCreated) Name() string {
	return EventProjectCreated
}

func NewProjectCreatedEvent(project Project) ProjectCreated {
	return ProjectCreated{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      project.ID,
		OrganizationID: project.OrganizationID,
		ProjectName:    project.Name,
		PathKey:        project.PathKey,
		FullPath:       project.FullPath,
		Visibility:     project.Visibility,
		DefaultBranch:  project.DefaultBranch,
	}
}

type ProjectDeleted struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	OrganizationID int64  `json:"organization_id"`
	ProjectName    string `json:"name"`
	PathKey        string `json:"path_key"`
	FullPath       string `json:"full_path"`
	Status         string `json:"status"`
	DeletedAt      string `json:"deleted_at,omitempty"`
}

func (ProjectDeleted) Name() string {
	return EventProjectDeleted
}

func NewProjectDeletedEvent(project Project) ProjectDeleted {
	deletedAt := ""
	if !project.DeletedAt.IsZero() {
		deletedAt = project.DeletedAt.UTC().Format("2006-01-02T15:04:05Z")
	}
	return ProjectDeleted{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      project.ID,
		OrganizationID: project.OrganizationID,
		ProjectName:    project.Name,
		PathKey:        project.PathKey,
		FullPath:       project.FullPath,
		Status:         project.Status,
		DeletedAt:      deletedAt,
	}
}
