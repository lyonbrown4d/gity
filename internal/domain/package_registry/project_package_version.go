package packageregistry

import "time"

type ProjectPackageVersion struct {
	ID               int64     `dbx:"id"`
	ProjectPackageID int64     `dbx:"project_package_id"`
	Version          string    `dbx:"version"`
	Status           string    `dbx:"status"`
	CreatedAt        time.Time `dbx:"created_at"`
	UpdatedAt        time.Time `dbx:"updated_at"`
}
