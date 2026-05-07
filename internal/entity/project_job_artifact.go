package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
)

type ProjectJobArtifact struct {
	ID           int64     `dbx:"id" json:"id"`
	ProjectID    int64     `dbx:"project_id" json:"project_id"`
	ProjectJobID int64     `dbx:"project_job_id" json:"project_job_id"`
	Name         string    `dbx:"name" json:"name"`
	FileName     string    `dbx:"file_name" json:"file_name"`
	FilePath     string    `dbx:"file_path" json:"file_path"`
	ContentType  string    `dbx:"content_type" json:"content_type"`
	ByteSize     int64     `dbx:"byte_size" json:"byte_size"`
	StorageKey   string    `dbx:"storage_key" json:"storage_key"`
	CreatedAt    time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at" json:"updated_at"`
}

type ProjectJobArtifactSchemaDef struct {
	dbx.Schema[ProjectJobArtifact]
	ID           dbx.IDColumn[ProjectJobArtifact, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID    dbx.Column[ProjectJobArtifact, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID dbx.Column[ProjectJobArtifact, int64]                    `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         dbx.Column[ProjectJobArtifact, string]                   `dbx:"name"`
	FileName     dbx.Column[ProjectJobArtifact, string]                   `dbx:"file_name"`
	FilePath     dbx.Column[ProjectJobArtifact, string]                   `dbx:"file_path"`
	ContentType  dbx.Column[ProjectJobArtifact, string]                   `dbx:"content_type"`
	ByteSize     dbx.Column[ProjectJobArtifact, int64]                    `dbx:"byte_size"`
	StorageKey   dbx.Column[ProjectJobArtifact, string]                   `dbx:"storage_key"`
	CreatedAt    dbx.Column[ProjectJobArtifact, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    dbx.Column[ProjectJobArtifact, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobArtifactSchema = dbx.MustSchema("project_job_artifacts", ProjectJobArtifactSchemaDef{})
