package dbschema

import (
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPipelineJobSchemaDef struct {
	schema.Schema[cidomain.ProjectPipelineJob]
	ID           column.IDColumn[cidomain.ProjectPipelineJob, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[cidomain.ProjectPipelineJob, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	PipelineID   column.Column[cidomain.ProjectPipelineJob, int64]                      `dbx:"pipeline_id,index,ref=project_pipelines.id,ondelete=cascade"`
	ProjectJobID column.Column[cidomain.ProjectPipelineJob, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"name"`
	Stage        column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"stage,index"`
	Needs        column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"needs,type=TEXT,null"`
	Image        column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"image,null"`
	Script       column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"script,type=TEXT"`
	Artifacts    column.Column[cidomain.ProjectPipelineJob, string]                     `dbx:"artifacts,type=TEXT,null"`
	SortOrder    column.Column[cidomain.ProjectPipelineJob, int]                        `dbx:"sort_order,index"`
	CreatedAt    column.Column[cidomain.ProjectPipelineJob, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[cidomain.ProjectPipelineJob, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPipelineJobSchema = schema.MustSchema("project_pipeline_jobs", ProjectPipelineJobSchemaDef{})
