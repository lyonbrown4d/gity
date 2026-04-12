package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
)

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

type ProjectIssueAttachmentSchemaDef struct {
	dbx.Schema[ProjectIssueAttachment]
	ID               dbx.IDColumn[ProjectIssueAttachment, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID   dbx.Column[ProjectIssueAttachment, int64]                    `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	UploadedByUserID dbx.Column[ProjectIssueAttachment, int64]                    `dbx:"uploaded_by_user_id,index,ref=users.id,ondelete=restrict"`
	FileName         dbx.Column[ProjectIssueAttachment, string]                   `dbx:"file_name"`
	ContentType      dbx.Column[ProjectIssueAttachment, string]                   `dbx:"content_type,null"`
	ByteSize         dbx.Column[ProjectIssueAttachment, int64]                    `dbx:"byte_size"`
	StorageKey       dbx.Column[ProjectIssueAttachment, string]                   `dbx:"storage_key,unique"`
	CreatedAt        dbx.Column[ProjectIssueAttachment, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        dbx.Column[ProjectIssueAttachment, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueAttachmentSchema = dbx.MustSchema("project_issue_attachments", ProjectIssueAttachmentSchemaDef{})
