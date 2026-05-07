package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectBranchProtection struct {
	ID         int64     `dbx:"id"`
	ProjectID  int64     `dbx:"project_id"`
	BranchName string    `dbx:"branch_name"`
	CreatedAt  time.Time `dbx:"created_at"`
	UpdatedAt  time.Time `dbx:"updated_at"`
}

type ProjectBranchProtectionSchemaDef struct {
	schema.Schema[ProjectBranchProtection]
	ID                  column.IDColumn[ProjectBranchProtection, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID           column.Column[ProjectBranchProtection, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	BranchName          column.Column[ProjectBranchProtection, string]                     `dbx:"branch_name,index"`
	CreatedAt           column.Column[ProjectBranchProtection, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt           column.Column[ProjectBranchProtection, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	ProjectBranchUnique schema.Unique[ProjectBranchProtection]                             `idx:"columns=project_id,branch_name"`
}

var ProjectBranchProtectionSchema = schema.MustSchema("project_branch_protections", ProjectBranchProtectionSchemaDef{})
