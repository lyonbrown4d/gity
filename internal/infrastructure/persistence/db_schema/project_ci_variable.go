package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type ProjectCIVariableSchemaDef struct {
	schema.Schema[cidomain.ProjectCIVariable]
	ID        column.IDColumn[cidomain.ProjectCIVariable, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID column.Column[cidomain.ProjectCIVariable, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Key       column.Column[cidomain.ProjectCIVariable, string]                     `dbx:"key,index"`
	Value     column.Column[cidomain.ProjectCIVariable, string]                     `dbx:"value"`
	Masked    column.Column[cidomain.ProjectCIVariable, int]                        `dbx:"masked,index"`
	Protected column.Column[cidomain.ProjectCIVariable, int]                        `dbx:"protected,index"`
	CreatedAt column.Column[cidomain.ProjectCIVariable, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[cidomain.ProjectCIVariable, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`

	UniqueProjectCIVariable schema.Unique[cidomain.ProjectCIVariable] `idx:"columns=project_id|key"`
}

var ProjectCIVariableSchema = schema.MustSchema("project_ci_variables", ProjectCIVariableSchemaDef{})
