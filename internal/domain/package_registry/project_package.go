// Package packageregistry defines package registry domain models.
package packageregistry

import "time"

type ProjectPackage struct {
	ID        int64     `dbx:"id"`
	ProjectID int64     `dbx:"project_id"`
	Type      string    `dbx:"type"`
	Name      string    `dbx:"name"`
	CreatedAt time.Time `dbx:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at"`
}
