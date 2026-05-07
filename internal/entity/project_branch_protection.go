package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
)

type ProjectBranchProtection struct {
	ID         int64     `dbx:"id"`
	ProjectID  int64     `dbx:"project_id"`
	BranchName string    `dbx:"branch_name"`
	CreatedAt  time.Time `dbx:"created_at"`
	UpdatedAt  time.Time `dbx:"updated_at"`
}

type ProjectBranchProtectionSchemaDef struct {
	dbx.Schema[ProjectBranchProtection]
	ID                  dbx.IDColumn[ProjectBranchProtection, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID           dbx.Column[ProjectBranchProtection, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	BranchName          dbx.Column[ProjectBranchProtection, string]                   `dbx:"branch_name,index"`
	CreatedAt           dbx.Column[ProjectBranchProtection, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt           dbx.Column[ProjectBranchProtection, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
	ProjectBranchUnique dbx.Unique[ProjectBranchProtection]                           `idx:"columns=project_id,branch_name"`
}

var ProjectBranchProtectionSchema = dbx.MustSchema("project_branch_protections", ProjectBranchProtectionSchemaDef{})
