package dbschema

import (
	"time"

	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectIssueAssigneeSchemaDef struct {
	schema.Schema[issuedomain.ProjectIssueAssignee]
	ID             column.IDColumn[issuedomain.ProjectIssueAssignee, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectIssueID column.Column[issuedomain.ProjectIssueAssignee, int64]                      `dbx:"project_issue_id,index,ref=project_issues.id,ondelete=cascade"`
	UserID         column.Column[issuedomain.ProjectIssueAssignee, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	CreatedAt      column.Column[issuedomain.ProjectIssueAssignee, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[issuedomain.ProjectIssueAssignee, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`

	UniqueProjectIssueUser schema.Unique[issuedomain.ProjectIssueAssignee] `idx:"columns=project_issue_id|user_id"`
}

var ProjectIssueAssigneeSchema = schema.MustSchema("project_issue_assignees", ProjectIssueAssigneeSchemaDef{})
