package issue

import "time"

type ProjectIssueComment struct {
	ID             int64     `dbx:"id"`
	ProjectIssueID int64     `dbx:"project_issue_id"`
	AuthorUserID   int64     `dbx:"author_user_id"`
	Body           string    `dbx:"body"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
