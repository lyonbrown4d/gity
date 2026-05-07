package project

import domainevent "github.com/DaiYuANg/gity/internal/domain/event"

const (
	EventProjectCreated = "project.created"
	EventProjectDeleted = "project.deleted"
)

type ProjectCreated struct {
	domainevent.Metadata
	ProjectID     int64  `json:"project_id"`
	NamespaceID   int64  `json:"namespace_id"`
	ProjectName   string `json:"name"`
	PathKey       string `json:"path_key"`
	FullPath      string `json:"full_path"`
	Visibility    string `json:"visibility"`
	DefaultBranch string `json:"default_branch"`
}

func (ProjectCreated) Name() string {
	return EventProjectCreated
}

func NewProjectCreatedEvent(project Project) ProjectCreated {
	return ProjectCreated{
		Metadata:      domainevent.NewMetadata(),
		ProjectID:     project.ID,
		NamespaceID:   project.NamespaceID,
		ProjectName:   project.Name,
		PathKey:       project.PathKey,
		FullPath:      project.FullPath,
		Visibility:    project.Visibility,
		DefaultBranch: project.DefaultBranch,
	}
}

type ProjectDeleted struct {
	domainevent.Metadata
	ProjectID int64 `json:"project_id"`
}

func (ProjectDeleted) Name() string {
	return EventProjectDeleted
}

func NewProjectDeletedEvent(projectID int64) ProjectDeleted {
	return ProjectDeleted{
		Metadata:  domainevent.NewMetadata(),
		ProjectID: projectID,
	}
}
