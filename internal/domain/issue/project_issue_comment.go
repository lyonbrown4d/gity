package issue

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectIssueComment struct {
	ID             int64     `dbx:"id"`
	ProjectIssueID int64     `dbx:"project_issue_id"`
	AuthorUserID   int64     `dbx:"author_user_id"`
	Body           string    `dbx:"body"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}

type ProjectIssueCommentSchemaDef struct {
	schema.Schema[ProjectIssueComment]
	ID             column.IDColumn[ProjectIssueComment, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID column.Column[ProjectIssueComment, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	AuthorUserID   column.Column[ProjectIssueComment, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Body           column.Column[ProjectIssueComment, string]                     `dbx:"body"`
	CreatedAt      column.Column[ProjectIssueComment, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[ProjectIssueComment, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueCommentSchema = schema.MustSchema("project_issue_comments", ProjectIssueCommentSchemaDef{})
