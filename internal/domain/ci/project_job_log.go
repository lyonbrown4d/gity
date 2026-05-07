package ci

import "time"

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
