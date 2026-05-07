package project

import "time"

type ProjectBranchProtection struct {
	ID         int64     `dbx:"id"`
	ProjectID  int64     `dbx:"project_id"`
	BranchName string    `dbx:"branch_name"`
	CreatedAt  time.Time `dbx:"created_at"`
	UpdatedAt  time.Time `dbx:"updated_at"`
}
