package project

import "time"

const (
	ProjectStatusActive        = "active"
	ProjectStatusPendingDelete = "pending_delete"
)

type Project struct {
	ID             int64     `dbx:"id"`
	OrganizationID int64     `dbx:"organization_id"`
	Name           string    `dbx:"name"`
	PathKey        string    `dbx:"path_key"`
	FullPath       string    `dbx:"full_path"`
	Visibility     string    `dbx:"visibility"`
	Description    string    `dbx:"description"`
	DefaultBranch  string    `dbx:"default_branch"`
	Status         string    `dbx:"status"`
	DeletedAt      time.Time `dbx:"deleted_at"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}

func (p Project) IsPendingDelete() bool {
	return p.Status == ProjectStatusPendingDelete
}
