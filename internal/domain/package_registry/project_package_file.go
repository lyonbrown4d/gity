package packageregistry

import "time"

type ProjectPackageFile struct {
	ID                      int64     `dbx:"id"`
	ProjectPackageVersionID int64     `dbx:"project_package_version_id"`
	FileName                string    `dbx:"file_name"`
	FilePath                string    `dbx:"file_path"`
	ContentType             string    `dbx:"content_type"`
	ByteSize                int64     `dbx:"byte_size"`
	StorageKey              string    `dbx:"storage_key"`
	CreatedAt               time.Time `dbx:"created_at"`
	UpdatedAt               time.Time `dbx:"updated_at"`
}
