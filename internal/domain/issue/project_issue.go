package issue

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectIssue struct {
	ID           int64     `dbx:"id"`
	ProjectID    int64     `dbx:"project_id"`
	IID          int64     `dbx:"iid"`
	AuthorUserID int64     `dbx:"author_user_id"`
	Title        string    `dbx:"title"`
	Description  string    `dbx:"description"`
	State        string    `dbx:"state"`
	CreatedAt    time.Time `dbx:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at"`
}

type ProjectIssueSchemaDef struct {
	schema.Schema[ProjectIssue]
	ID           column.IDColumn[ProjectIssue, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[ProjectIssue, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID          column.Column[ProjectIssue, int64]                      `dbx:"iid,index"`
	AuthorUserID column.Column[ProjectIssue, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Title        column.Column[ProjectIssue, string]                     `dbx:"title"`
	Description  column.Column[ProjectIssue, string]                     `dbx:"description,null"`
	State        column.Column[ProjectIssue, string]                     `dbx:"state,index"`
	CreatedAt    column.Column[ProjectIssue, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[ProjectIssue, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueSchema = schema.MustSchema("project_issues", ProjectIssueSchemaDef{})
