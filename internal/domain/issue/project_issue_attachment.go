package issue

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
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
	schema.Schema[ProjectIssueAttachment]
	ID               column.IDColumn[ProjectIssueAttachment, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID   column.Column[ProjectIssueAttachment, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	UploadedByUserID column.Column[ProjectIssueAttachment, int64]                      `dbx:"uploaded_by_user_id,index,ref=users.id,ondelete=restrict"`
	FileName         column.Column[ProjectIssueAttachment, string]                     `dbx:"file_name"`
	ContentType      column.Column[ProjectIssueAttachment, string]                     `dbx:"content_type,null"`
	ByteSize         column.Column[ProjectIssueAttachment, int64]                      `dbx:"byte_size"`
	StorageKey       column.Column[ProjectIssueAttachment, string]                     `dbx:"storage_key,unique"`
	CreatedAt        column.Column[ProjectIssueAttachment, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        column.Column[ProjectIssueAttachment, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueAttachmentSchema = schema.MustSchema("project_issue_attachments", ProjectIssueAttachmentSchemaDef{})
