package issue

import "time"

type ProjectIssue struct {
	ID           int64     `dbx:"id"`
	ProjectID    int64     `dbx:"project_id"`
	IID          int64     `dbx:"iid"`
	AuthorUserID int64     `dbx:"author_user_id"`
	Title        string    `dbx:"title"`
	Description  string    `dbx:"description"`
	State        string    `dbx:"state"`
	CreatedAt    time.Time `dbx:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at"`
}
