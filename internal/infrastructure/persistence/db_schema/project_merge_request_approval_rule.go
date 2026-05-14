package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

type ProjectMergeRequestApprovalRuleSchemaDef struct {
	schema.Schema[mergedomain.ProjectMergeRequestApprovalRule]
	ID                column.IDColumn[mergedomain.ProjectMergeRequestApprovalRule, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID         column.Column[mergedomain.ProjectMergeRequestApprovalRule, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Name              column.Column[mergedomain.ProjectMergeRequestApprovalRule, string]                     `dbx:"name,index"`
	TargetBranch      column.Column[mergedomain.ProjectMergeRequestApprovalRule, string]                     `dbx:"target_branch,index"`
	ApprovalsRequired column.Column[mergedomain.ProjectMergeRequestApprovalRule, int]                        `dbx:"approvals_required"`
	EligibleUserIDs   column.Column[mergedomain.ProjectMergeRequestApprovalRule, string]                     `dbx:"eligible_user_ids"`
	CodeOwner         column.Column[mergedomain.ProjectMergeRequestApprovalRule, int]                        `dbx:"code_owner,index"`
	CreatedAt         column.Column[mergedomain.ProjectMergeRequestApprovalRule, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt         column.Column[mergedomain.ProjectMergeRequestApprovalRule, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMergeRequestApprovalRuleSchema = schema.MustSchema("project_merge_request_approval_rules", ProjectMergeRequestApprovalRuleSchemaDef{})
