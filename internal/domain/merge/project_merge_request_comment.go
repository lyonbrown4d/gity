package merge

import "time"

type ProjectMergeRequestComment struct {
	ID             int64     `dbx:"id"`
	MergeRequestID int64     `dbx:"merge_request_id"`
	AuthorUserID   int64     `dbx:"author_user_id"`
	Body           string    `dbx:"body"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
