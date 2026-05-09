package dbschema

import (
	"time"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectBranchProtectionSchemaDef struct {
	schema.Schema[projectdomain.ProjectBranchProtection]
	ID                     column.IDColumn[projectdomain.ProjectBranchProtection, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID              column.Column[projectdomain.ProjectBranchProtection, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	BranchName             column.Column[projectdomain.ProjectBranchProtection, string]                     `dbx:"branch_name,index"`
	RuleType               column.Column[projectdomain.ProjectBranchProtection, string]                     `dbx:"rule_type,index"`
	PushAccessLevel        column.Column[projectdomain.ProjectBranchProtection, string]                     `dbx:"push_access_level"`
	MergeAccessLevel       column.Column[projectdomain.ProjectBranchProtection, string]                     `dbx:"merge_access_level"`
	RequireMergeRequest    column.Column[projectdomain.ProjectBranchProtection, int]                        `dbx:"require_merge_request"`
	RequirePipelineSuccess column.Column[projectdomain.ProjectBranchProtection, int]                        `dbx:"require_pipeline_success"`
	AllowForcePush         column.Column[projectdomain.ProjectBranchProtection, int]                        `dbx:"allow_force_push"`
	AllowDelete            column.Column[projectdomain.ProjectBranchProtection, int]                        `dbx:"allow_delete"`
	CreatedAt              column.Column[projectdomain.ProjectBranchProtection, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt              column.Column[projectdomain.ProjectBranchProtection, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	ProjectBranchUnique    schema.Unique[projectdomain.ProjectBranchProtection]                             `idx:"columns=project_id|branch_name"`
}

var ProjectBranchProtectionSchema = schema.MustSchema("project_branch_protections", ProjectBranchProtectionSchemaDef{})
