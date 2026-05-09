package project

import "time"

type Project struct {
	ID             int64     `dbx:"id"`
	OrganizationID int64     `dbx:"organization_id"`
	Name           string    `dbx:"name"`
	PathKey        string    `dbx:"path_key"`
	FullPath       string    `dbx:"full_path"`
	Visibility     string    `dbx:"visibility"`
	Description    string    `dbx:"description"`
	DefaultBranch  string    `dbx:"default_branch"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
