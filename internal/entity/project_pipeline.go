package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectPipeline]
	ID            dbx.IDColumn[ProjectPipeline, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID     dbx.Column[ProjectPipeline, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID           dbx.Column[ProjectPipeline, int64]                    `dbx:"iid,index"`
	Name          dbx.Column[ProjectPipeline, string]                   `dbx:"name"`
	Source        dbx.Column[ProjectPipeline, string]                   `dbx:"source,index"`
	RefName       dbx.Column[ProjectPipeline, string]                   `dbx:"ref_name,index,null"`
	CommitSHA     dbx.Column[ProjectPipeline, string]                   `dbx:"commit_sha,index,null"`
	Status        dbx.Column[ProjectPipeline, string]                   `dbx:"status,index"`
	ConfigSource  dbx.Column[ProjectPipeline, string]                   `dbx:"config_source"`
	ConfigContent dbx.Column[ProjectPipeline, string]                   `dbx:"config_content,type=TEXT"`
	CreatedAt     dbx.Column[ProjectPipeline, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     dbx.Column[ProjectPipeline, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt     dbx.Column[ProjectPipeline, time.Time]                `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt    dbx.Column[ProjectPipeline, time.Time]                `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectPipelineSchema = dbx.MustSchema("project_pipelines", ProjectPipelineSchemaDef{})
