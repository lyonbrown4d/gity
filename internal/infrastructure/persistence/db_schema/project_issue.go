package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
)

type ProjectIssueSchemaDef struct {
	schema.Schema[issuedomain.ProjectIssue]
	ID           column.IDColumn[issuedomain.ProjectIssue, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[issuedomain.ProjectIssue, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID          column.Column[issuedomain.ProjectIssue, int64]                      `dbx:"iid,index"`
	AuthorUserID column.Column[issuedomain.ProjectIssue, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Title        column.Column[issuedomain.ProjectIssue, string]                     `dbx:"title"`
	Description  column.Column[issuedomain.ProjectIssue, string]                     `dbx:"description,null"`
	State        column.Column[issuedomain.ProjectIssue, string]                     `dbx:"state,index"`
	CreatedAt    column.Column[issuedomain.ProjectIssue, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[issuedomain.ProjectIssue, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectIssueSchema = schema.MustSchema("project_issues", ProjectIssueSchemaDef{})
