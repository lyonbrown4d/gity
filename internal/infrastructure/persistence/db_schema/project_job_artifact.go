package dbschema

import (
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectJobArtifactSchemaDef struct {
	schema.Schema[cidomain.ProjectJobArtifact]
	ID           column.IDColumn[cidomain.ProjectJobArtifact, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[cidomain.ProjectJobArtifact, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID column.Column[cidomain.ProjectJobArtifact, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         column.Column[cidomain.ProjectJobArtifact, string]                     `dbx:"name"`
	FileName     column.Column[cidomain.ProjectJobArtifact, string]                     `dbx:"file_name"`
	FilePath     column.Column[cidomain.ProjectJobArtifact, string]                     `dbx:"file_path"`
	ContentType  column.Column[cidomain.ProjectJobArtifact, string]                     `dbx:"content_type"`
	ByteSize     column.Column[cidomain.ProjectJobArtifact, int64]                      `dbx:"byte_size"`
	StorageKey   column.Column[cidomain.ProjectJobArtifact, string]                     `dbx:"storage_key"`
	CreatedAt    column.Column[cidomain.ProjectJobArtifact, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[cidomain.ProjectJobArtifact, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobArtifactSchema = schema.MustSchema("project_job_artifacts", ProjectJobArtifactSchemaDef{})
