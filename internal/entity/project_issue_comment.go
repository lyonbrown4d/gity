package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectIssueComment]
	ID             dbx.IDColumn[ProjectIssueComment, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID dbx.Column[ProjectIssueComment, int64]                    `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	AuthorUserID   dbx.Column[ProjectIssueComment, int64]                    `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Body           dbx.Column[ProjectIssueComment, string]                   `dbx:"body"`
	CreatedAt      dbx.Column[ProjectIssueComment, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      dbx.Column[ProjectIssueComment, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueCommentSchema = dbx.MustSchema("project_issue_comments", ProjectIssueCommentSchemaDef{})
