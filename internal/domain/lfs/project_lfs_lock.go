// Package lfs defines Git LFS domain models.
package lfs

import "time"

type ProjectLFSLock struct {
	ID          int64     `dbx:"id"`
	ProjectID   int64     `dbx:"project_id"`
	OwnerUserID int64     `dbx:"owner_user_id"`
	Path        string    `dbx:"path"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}
