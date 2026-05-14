package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

type ProjectMergeRequestApprovalSchemaDef struct {
	schema.Schema[mergedomain.ProjectMergeRequestApproval]
	ID             column.IDColumn[mergedomain.ProjectMergeRequestApproval, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	MergeRequestID column.Column[mergedomain.ProjectMergeRequestApproval, int64]                      `dbx:"merge_request_id,index,ref=project_merge_requests.id,ondelete=cascade"`
	UserID         column.Column[mergedomain.ProjectMergeRequestApproval, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	CreatedAt      column.Column[mergedomain.ProjectMergeRequestApproval, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[mergedomain.ProjectMergeRequestApproval, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`

	UniqueMergeRequestApproval schema.Unique[mergedomain.ProjectMergeRequestApproval] `idx:"columns=merge_request_id|user_id"`
}

var ProjectMergeRequestApprovalSchema = schema.MustSchema("project_merge_request_approvals", ProjectMergeRequestApprovalSchemaDef{})
