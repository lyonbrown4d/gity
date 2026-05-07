package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPipeline struct {
	ID            int64     `dbx:"id" json:"id"`
	ProjectID     int64     `dbx:"project_id" json:"project_id"`
	IID           int64     `dbx:"iid" json:"iid"`
	Name          string    `dbx:"name" json:"name"`
	Source        string    `dbx:"source" json:"source"`
	RefName       string    `dbx:"ref_name" json:"ref_name"`
	CommitSHA     string    `dbx:"commit_sha" json:"commit_sha"`
	Status        string    `dbx:"status" json:"status"`
	ConfigSource  string    `dbx:"config_source" json:"config_source"`
	ConfigContent string    `dbx:"config_content" json:"config_content,omitempty"`
	CreatedAt     time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt     time.Time `dbx:"updated_at" json:"updated_at"`
	StartedAt     time.Time `dbx:"started_at" json:"started_at"`
	FinishedAt    time.Time `dbx:"finished_at" json:"finished_at"`
}

type ProjectPipelineSchemaDef struct {
	schema.Schema[ProjectPipeline]
	ID            column.IDColumn[ProjectPipeline, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID     column.Column[ProjectPipeline, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID           column.Column[ProjectPipeline, int64]                      `dbx:"iid,index"`
	Name          column.Column[ProjectPipeline, string]                     `dbx:"name"`
	Source        column.Column[ProjectPipeline, string]                     `dbx:"source,index"`
	RefName       column.Column[ProjectPipeline, string]                     `dbx:"ref_name,index,null"`
	CommitSHA     column.Column[ProjectPipeline, string]                     `dbx:"commit_sha,index,null"`
	Status        column.Column[ProjectPipeline, string]                     `dbx:"status,index"`
	ConfigSource  column.Column[ProjectPipeline, string]                     `dbx:"config_source"`
	ConfigContent column.Column[ProjectPipeline, string]                     `dbx:"config_content,type=TEXT"`
	CreatedAt     column.Column[ProjectPipeline, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     column.Column[ProjectPipeline, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt     column.Column[ProjectPipeline, time.Time]                  `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt    column.Column[ProjectPipeline, time.Time]                  `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectPipelineSchema = schema.MustSchema("project_pipelines", ProjectPipelineSchemaDef{})
