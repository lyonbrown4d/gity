package issue

import "time"

type ProjectIssueAttachment struct {
	ID               int64     `dbx:"id"`
	ProjectIssueID   int64     `dbx:"project_issue_id"`
	UploadedByUserID int64     `dbx:"uploaded_by_user_id"`
	FileName         string    `dbx:"file_name"`
	ContentType      string    `dbx:"content_type"`
	ByteSize         int64     `dbx:"byte_size"`
	StorageKey       string    `dbx:"storage_key"`
	CreatedAt        time.Time `dbx:"created_at"`
	UpdatedAt        time.Time `dbx:"updated_at"`
}
