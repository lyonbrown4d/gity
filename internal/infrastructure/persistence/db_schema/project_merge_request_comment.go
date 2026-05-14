package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

type ProjectMergeRequestCommentSchemaDef struct {
	schema.Schema[mergedomain.ProjectMergeRequestComment]
	ID             column.IDColumn[mergedomain.ProjectMergeRequestComment, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	MergeRequestID column.Column[mergedomain.ProjectMergeRequestComment, int64]                      `dbx:"merge_request_id,index,ref=project_merge_requests.id,ondelete=cascade"`
	AuthorUserID   column.Column[mergedomain.ProjectMergeRequestComment, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Body           column.Column[mergedomain.ProjectMergeRequestComment, string]                     `dbx:"body"`
	CreatedAt      column.Column[mergedomain.ProjectMergeRequestComment, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[mergedomain.ProjectMergeRequestComment, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMergeRequestCommentSchema = schema.MustSchema("project_merge_request_comments", ProjectMergeRequestCommentSchemaDef{})
