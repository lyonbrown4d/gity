package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectJob]
	ID          dbx.IDColumn[ProjectJob, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID   dbx.Column[ProjectJob, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Kind        dbx.Column[ProjectJob, string]                   `dbx:"kind,index"`
	Status      dbx.Column[ProjectJob, string]                   `dbx:"status,index"`
	Payload     dbx.Column[ProjectJob, string]                   `dbx:"payload,type=TEXT,null"`
	Result      dbx.Column[ProjectJob, string]                   `dbx:"result,type=TEXT,null"`
	Attempts    dbx.Column[ProjectJob, int]                      `dbx:"attempts"`
	MaxAttempts dbx.Column[ProjectJob, int]                      `dbx:"max_attempts"`
	RunAfter    dbx.Column[ProjectJob, time.Time]                `dbx:"run_after,type=TIMESTAMP,index"`
	LockedBy    dbx.Column[ProjectJob, string]                   `dbx:"locked_by,index,null"`
	LockedUntil dbx.Column[ProjectJob, time.Time]                `dbx:"locked_until,type=TIMESTAMP,index"`
	LastError   dbx.Column[ProjectJob, string]                   `dbx:"last_error,type=TEXT,null"`
	CreatedAt   dbx.Column[ProjectJob, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   dbx.Column[ProjectJob, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt   dbx.Column[ProjectJob, time.Time]                `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt  dbx.Column[ProjectJob, time.Time]                `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectJobSchema = dbx.MustSchema("project_jobs", ProjectJobSchemaDef{})
