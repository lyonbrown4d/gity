package dbschema

import (
	"time"

	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectMergeRequestSchemaDef struct {
	schema.Schema[mergedomain.ProjectMergeRequest]
	ID           column.IDColumn[mergedomain.ProjectMergeRequest, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[mergedomain.ProjectMergeRequest, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID          column.Column[mergedomain.ProjectMergeRequest, int64]                      `dbx:"iid,index"`
	AuthorUserID column.Column[mergedomain.ProjectMergeRequest, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Title        column.Column[mergedomain.ProjectMergeRequest, string]                     `dbx:"title"`
	Description  column.Column[mergedomain.ProjectMergeRequest, string]                     `dbx:"description,null"`
	State        column.Column[mergedomain.ProjectMergeRequest, string]                     `dbx:"state,index"`
	SourceBranch column.Column[mergedomain.ProjectMergeRequest, string]                     `dbx:"source_branch,index"`
	TargetBranch column.Column[mergedomain.ProjectMergeRequest, string]                     `dbx:"target_branch,index"`
	CreatedAt    column.Column[mergedomain.ProjectMergeRequest, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[mergedomain.ProjectMergeRequest, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMergeRequestSchema = schema.MustSchema("project_merge_requests", ProjectMergeRequestSchemaDef{})
