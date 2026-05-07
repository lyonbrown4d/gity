package ci

import "time"

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
