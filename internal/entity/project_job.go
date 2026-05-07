package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectJob struct {
	ID          int64     `dbx:"id"`
	ProjectID   int64     `dbx:"project_id"`
	Kind        string    `dbx:"kind"`
	Status      string    `dbx:"status"`
	Payload     string    `dbx:"payload"`
	Result      string    `dbx:"result"`
	Attempts    int       `dbx:"attempts"`
	MaxAttempts int       `dbx:"max_attempts"`
	RunAfter    time.Time `dbx:"run_after"`
	LockedBy    string    `dbx:"locked_by"`
	LockedUntil time.Time `dbx:"locked_until"`
	LastError   string    `dbx:"last_error"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
	StartedAt   time.Time `dbx:"started_at"`
	FinishedAt  time.Time `dbx:"finished_at"`
}

type ProjectJobSchemaDef struct {
	schema.Schema[ProjectJob]
	ID          column.IDColumn[ProjectJob, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID   column.Column[ProjectJob, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Kind        column.Column[ProjectJob, string]                     `dbx:"kind,index"`
	Status      column.Column[ProjectJob, string]                     `dbx:"status,index"`
	Payload     column.Column[ProjectJob, string]                     `dbx:"payload,type=TEXT,null"`
	Result      column.Column[ProjectJob, string]                     `dbx:"result,type=TEXT,null"`
	Attempts    column.Column[ProjectJob, int]                        `dbx:"attempts"`
	MaxAttempts column.Column[ProjectJob, int]                        `dbx:"max_attempts"`
	RunAfter    column.Column[ProjectJob, time.Time]                  `dbx:"run_after,type=TIMESTAMP,index"`
	LockedBy    column.Column[ProjectJob, string]                     `dbx:"locked_by,index,null"`
	LockedUntil column.Column[ProjectJob, time.Time]                  `dbx:"locked_until,type=TIMESTAMP,index"`
	LastError   column.Column[ProjectJob, string]                     `dbx:"last_error,type=TEXT,null"`
	CreatedAt   column.Column[ProjectJob, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[ProjectJob, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt   column.Column[ProjectJob, time.Time]                  `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt  column.Column[ProjectJob, time.Time]                  `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectJobSchema = schema.MustSchema("project_jobs", ProjectJobSchemaDef{})
