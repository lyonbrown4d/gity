package issue

import "time"

type ProjectIssueAssignee struct {
	ID             int64     `dbx:"id"`
	ProjectIssueID int64     `dbx:"project_issue_id"`
	UserID         int64     `dbx:"user_id"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
