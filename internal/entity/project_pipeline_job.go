package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectPipelineJob]
	ID           dbx.IDColumn[ProjectPipelineJob, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID    dbx.Column[ProjectPipelineJob, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	PipelineID   dbx.Column[ProjectPipelineJob, int64]                    `dbx:"pipeline_id,index,ref=project_pipelines.id,ondelete=cascade"`
	ProjectJobID dbx.Column[ProjectPipelineJob, int64]                    `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Name         dbx.Column[ProjectPipelineJob, string]                   `dbx:"name"`
	Stage        dbx.Column[ProjectPipelineJob, string]                   `dbx:"stage,index"`
	Needs        dbx.Column[ProjectPipelineJob, string]                   `dbx:"needs,type=TEXT,null"`
	Image        dbx.Column[ProjectPipelineJob, string]                   `dbx:"image,null"`
	Script       dbx.Column[ProjectPipelineJob, string]                   `dbx:"script,type=TEXT"`
	Artifacts    dbx.Column[ProjectPipelineJob, string]                   `dbx:"artifacts,type=TEXT,null"`
	SortOrder    dbx.Column[ProjectPipelineJob, int]                      `dbx:"sort_order,index"`
	CreatedAt    dbx.Column[ProjectPipelineJob, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    dbx.Column[ProjectPipelineJob, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPipelineJobSchema = dbx.MustSchema("project_pipeline_jobs", ProjectPipelineJobSchemaDef{})
