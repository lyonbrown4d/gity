package ci

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPipelineJob struct {
	ID           int64     `dbx:"id" json:"id"`
	ProjectID    int64     `dbx:"project_id" json:"project_id"`
	PipelineID   int64     `dbx:"pipeline_id" json:"pipeline_id"`
	ProjectJobID int64     `dbx:"project_job_id" json:"project_job_id"`
	Name         string    `dbx:"name" json:"name"`
	Stage        string    `dbx:"stage" json:"stage"`
	Needs        string    `dbx:"needs" json:"needs"`
	Image        string    `dbx:"image" json:"image"`
	Script       string    `dbx:"script" json:"script"`
	Artifacts    string    `dbx:"artifacts" json:"artifacts"`
	SortOrder    int       `dbx:"sort_order" json:"sort_order"`
	CreatedAt    time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at" json:"updated_at"`
}

type ProjectPipelineJobSchemaDef struct {
	schema.Schema[ProjectPipelineJob]
	ID           column.IDColumn[ProjectPipelineJob, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[ProjectPipelineJob, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	PipelineID   column.Column[ProjectPipelineJob, int64]                      `dbx:"pipeline_id,index,ref=project_pipelines.id,ondelete=cascade"`
	ProjectJobID column.Column[ProjectPipelineJob, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         column.Column[ProjectPipelineJob, string]                     `dbx:"name"`
	Stage        column.Column[ProjectPipelineJob, string]                     `dbx:"stage,index"`
	Needs        column.Column[ProjectPipelineJob, string]                     `dbx:"needs,type=TEXT,null"`
	Image        column.Column[ProjectPipelineJob, string]                     `dbx:"image,null"`
	Script       column.Column[ProjectPipelineJob, string]                     `dbx:"script,type=TEXT"`
	Artifacts    column.Column[ProjectPipelineJob, string]                     `dbx:"artifacts,type=TEXT,null"`
	SortOrder    column.Column[ProjectPipelineJob, int]                        `dbx:"sort_order,index"`
	CreatedAt    column.Column[ProjectPipelineJob, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[ProjectPipelineJob, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPipelineJobSchema = schema.MustSchema("project_pipeline_jobs", ProjectPipelineJobSchemaDef{})
