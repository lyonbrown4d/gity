package ci

import "time"

type ProjectJobArtifact struct {
	ID           int64     `dbx:"id"             json:"id"`
	ProjectID    int64     `dbx:"project_id"     json:"project_id"`
	ProjectJobID int64     `dbx:"project_job_id" json:"project_job_id"`
	Name         string    `dbx:"name"           json:"name"`
	FileName     string    `dbx:"file_name"      json:"file_name"`
	FilePath     string    `dbx:"file_path"      json:"file_path"`
	ContentType  string    `dbx:"content_type"   json:"content_type"`
	ByteSize     int64     `dbx:"byte_size"      json:"byte_size"`
	StorageKey   string    `dbx:"storage_key"    json:"storage_key"`
	CreatedAt    time.Time `dbx:"created_at"     json:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at"     json:"updated_at"`
}
