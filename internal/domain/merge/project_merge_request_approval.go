package merge

import "time"

type ProjectMergeRequestApproval struct {
	ID             int64     `dbx:"id"`
	MergeRequestID int64     `dbx:"merge_request_id"`
	UserID         int64     `dbx:"user_id"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
