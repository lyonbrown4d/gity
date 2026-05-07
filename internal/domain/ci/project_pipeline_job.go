package ci

import "time"

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
