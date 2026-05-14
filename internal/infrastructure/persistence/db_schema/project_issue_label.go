package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
)

type ProjectIssueLabelSchemaDef struct {
	schema.Schema[issuedomain.ProjectIssueLabel]
	ID             column.IDColumn[issuedomain.ProjectIssueLabel, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID column.Column[issuedomain.ProjectIssueLabel, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	Name           column.Column[issuedomain.ProjectIssueLabel, string]                     `dbx:"name,index"`
	Color          column.Column[issuedomain.ProjectIssueLabel, string]                     `dbx:"color,null"`
	CreatedAt      column.Column[issuedomain.ProjectIssueLabel, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[issuedomain.ProjectIssueLabel, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`

	UniqueProjectIssueLabel schema.Unique[issuedomain.ProjectIssueLabel] `idx:"columns=project_issue_id|name"`
}

var ProjectIssueLabelSchema = schema.MustSchema("project_issue_labels", ProjectIssueLabelSchemaDef{})
