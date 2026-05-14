package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
)

type ProjectIssueCommentSchemaDef struct {
	schema.Schema[issuedomain.ProjectIssueComment]
	ID             column.IDColumn[issuedomain.ProjectIssueComment, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID column.Column[issuedomain.ProjectIssueComment, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	AuthorUserID   column.Column[issuedomain.ProjectIssueComment, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Body           column.Column[issuedomain.ProjectIssueComment, string]                     `dbx:"body"`
	CreatedAt      column.Column[issuedomain.ProjectIssueComment, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[issuedomain.ProjectIssueComment, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueCommentSchema = schema.MustSchema("project_issue_comments", ProjectIssueCommentSchemaDef{})
