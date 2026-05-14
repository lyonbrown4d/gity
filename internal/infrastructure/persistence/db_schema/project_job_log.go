package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type ProjectJobLogSchemaDef struct {
	schema.Schema[cidomain.ProjectJobLog]
	ID              column.IDColumn[cidomain.ProjectJobLog, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID       column.Column[cidomain.ProjectJobLog, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	ProjectJobID    column.Column[cidomain.ProjectJobLog, int64]                      `dbx:"project_job_id,index,ref=project_jobs.id,ondelete=cascade"`
	Attempt         column.Column[cidomain.ProjectJobLog, int]                        `dbx:"attempt,index"`
	ExitCode        column.Column[cidomain.ProjectJobLog, int]                        `dbx:"exit_code"`
	Output          column.Column[cidomain.ProjectJobLog, string]                     `dbx:"output,type=TEXT"`
	OutputTruncated column.Column[cidomain.ProjectJobLog, int]                        `dbx:"output_truncated"`
	DurationMillis  column.Column[cidomain.ProjectJobLog, int64]                      `dbx:"duration_millis"`
	CreatedAt       column.Column[cidomain.ProjectJobLog, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[cidomain.ProjectJobLog, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectJobLogSchema = schema.MustSchema("project_job_logs", ProjectJobLogSchemaDef{})
