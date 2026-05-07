package dbschema

import (
	"time"

	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectIssueAttachmentSchemaDef struct {
	schema.Schema[issuedomain.ProjectIssueAttachment]
	ID               column.IDColumn[issuedomain.ProjectIssueAttachment, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID   column.Column[issuedomain.ProjectIssueAttachment, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	UploadedByUserID column.Column[issuedomain.ProjectIssueAttachment, int64]                      `dbx:"uploaded_by_user_id,index,ref=users.id,ondelete=restrict"`
	FileName         column.Column[issuedomain.ProjectIssueAttachment, string]                     `dbx:"file_name"`
	ContentType      column.Column[issuedomain.ProjectIssueAttachment, string]                     `dbx:"content_type,null"`
	ByteSize         column.Column[issuedomain.ProjectIssueAttachment, int64]                      `dbx:"byte_size"`
	StorageKey       column.Column[issuedomain.ProjectIssueAttachment, string]                     `dbx:"storage_key,unique"`
	CreatedAt        column.Column[issuedomain.ProjectIssueAttachment, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        column.Column[issuedomain.ProjectIssueAttachment, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueAttachmentSchema = schema.MustSchema("project_issue_attachments", ProjectIssueAttachmentSchemaDef{})
