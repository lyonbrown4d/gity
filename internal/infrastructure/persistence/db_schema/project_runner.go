package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type ProjectRunnerSchemaDef struct {
	schema.Schema[cidomain.ProjectRunner]
	ID            column.IDColumn[cidomain.ProjectRunner, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID     column.Column[cidomain.ProjectRunner, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Name          column.Column[cidomain.ProjectRunner, string]                     `dbx:"name,index"`
	Description   column.Column[cidomain.ProjectRunner, string]                     `dbx:"description,type=TEXT,null"`
	Tags          column.Column[cidomain.ProjectRunner, string]                     `dbx:"tags,index,null"`
	Token         column.Column[cidomain.ProjectRunner, string]                     `dbx:"token,unique"`
	Status        column.Column[cidomain.ProjectRunner, string]                     `dbx:"status,index"`
	Active        column.Column[cidomain.ProjectRunner, int]                        `dbx:"active,index"`
	LastContactAt column.Column[cidomain.ProjectRunner, time.Time]                  `dbx:"last_contact_at,type=TIMESTAMP,index"`
	CreatedAt     column.Column[cidomain.ProjectRunner, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     column.Column[cidomain.ProjectRunner, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectRunnerSchema = schema.MustSchema("project_runners", ProjectRunnerSchemaDef{})
