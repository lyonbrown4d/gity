package issue

import "time"

type ProjectIssueLabel struct {
	ID             int64     `dbx:"id"`
	ProjectIssueID int64     `dbx:"project_issue_id"`
	Name           string    `dbx:"name"`
	Color          string    `dbx:"color"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
