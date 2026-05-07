package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
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
	schema.Schema[ProjectJobArtifact]
	ID           column.IDColumn[ProjectJobArtifact, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[ProjectJobArtifact, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID column.Column[ProjectJobArtifact, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         column.Column[ProjectJobArtifact, string]                     `dbx:"name"`
	FileName     column.Column[ProjectJobArtifact, string]                     `dbx:"file_name"`
	FilePath     column.Column[ProjectJobArtifact, string]                     `dbx:"file_path"`
	ContentType  column.Column[ProjectJobArtifact, string]                     `dbx:"content_type"`
	ByteSize     column.Column[ProjectJobArtifact, int64]                      `dbx:"byte_size"`
	StorageKey   column.Column[ProjectJobArtifact, string]                     `dbx:"storage_key"`
	CreatedAt    column.Column[ProjectJobArtifact, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[ProjectJobArtifact, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobArtifactSchema = schema.MustSchema("project_job_artifacts", ProjectJobArtifactSchemaDef{})
