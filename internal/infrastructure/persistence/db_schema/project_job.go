package dbschema

import (
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectJobSchemaDef struct {
	schema.Schema[cidomain.ProjectJob]
	ID          column.IDColumn[cidomain.ProjectJob, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID   column.Column[cidomain.ProjectJob, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Kind        column.Column[cidomain.ProjectJob, string]                     `dbx:"kind,index"`
	Status      column.Column[cidomain.ProjectJob, string]                     `dbx:"status,index"`
	Payload     column.Column[cidomain.ProjectJob, string]                     `dbx:"payload,type=TEXT,null"`
	Result      column.Column[cidomain.ProjectJob, string]                     `dbx:"result,type=TEXT,null"`
	Attempts    column.Column[cidomain.ProjectJob, int]                        `dbx:"attempts"`
	MaxAttempts column.Column[cidomain.ProjectJob, int]                        `dbx:"max_attempts"`
	RunAfter    column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"run_after,type=TIMESTAMP,index"`
	LockedBy    column.Column[cidomain.ProjectJob, string]                     `dbx:"locked_by,index,null"`
	LockedUntil column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"locked_until,type=TIMESTAMP,index"`
	LastError   column.Column[cidomain.ProjectJob, string]                     `dbx:"last_error,type=TEXT,null"`
	CreatedAt   column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt   column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt  column.Column[cidomain.ProjectJob, time.Time]                  `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectJobSchema = schema.MustSchema("project_jobs", ProjectJobSchemaDef{})
