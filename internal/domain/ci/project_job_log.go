package ci

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectJobLog struct {
	ID              int64     `dbx:"id" json:"id"`
	ProjectID       int64     `dbx:"project_id" json:"project_id"`
	ProjectJobID    int64     `dbx:"project_job_id" json:"project_job_id"`
	Attempt         int       `dbx:"attempt" json:"attempt"`
	ExitCode        int       `dbx:"exit_code" json:"exit_code"`
	Output          string    `dbx:"output" json:"output"`
	OutputTruncated int       `dbx:"output_truncated" json:"output_truncated"`
	DurationMillis  int64     `dbx:"duration_millis" json:"duration_millis"`
	CreatedAt       time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt       time.Time `dbx:"updated_at" json:"updated_at"`
}

type ProjectJobLogSchemaDef struct {
	schema.Schema[ProjectJobLog]
	ID              column.IDColumn[ProjectJobLog, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID       column.Column[ProjectJobLog, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID    column.Column[ProjectJobLog, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Attempt         column.Column[ProjectJobLog, int]                        `dbx:"attempt,index"`
	ExitCode        column.Column[ProjectJobLog, int]                        `dbx:"exit_code"`
	Output          column.Column[ProjectJobLog, string]                     `dbx:"output,type=TEXT"`
	OutputTruncated column.Column[ProjectJobLog, int]                        `dbx:"output_truncated"`
	DurationMillis  column.Column[ProjectJobLog, int64]                      `dbx:"duration_millis"`
	CreatedAt       column.Column[ProjectJobLog, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[ProjectJobLog, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobLogSchema = schema.MustSchema("project_job_logs", ProjectJobLogSchemaDef{})
