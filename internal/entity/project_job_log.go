package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectJobLog]
	ID              dbx.IDColumn[ProjectJobLog, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID       dbx.Column[ProjectJobLog, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID    dbx.Column[ProjectJobLog, int64]                    `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Attempt         dbx.Column[ProjectJobLog, int]                      `dbx:"attempt,index"`
	ExitCode        dbx.Column[ProjectJobLog, int]                      `dbx:"exit_code"`
	Output          dbx.Column[ProjectJobLog, string]                   `dbx:"output,type=TEXT"`
	OutputTruncated dbx.Column[ProjectJobLog, int]                      `dbx:"output_truncated"`
	DurationMillis  dbx.Column[ProjectJobLog, int64]                    `dbx:"duration_millis"`
	CreatedAt       dbx.Column[ProjectJobLog, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       dbx.Column[ProjectJobLog, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobLogSchema = dbx.MustSchema("project_job_logs", ProjectJobLogSchemaDef{})
