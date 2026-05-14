package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
)

type ProjectMemberSchemaDef struct {
	schema.Schema[projectdomain.ProjectMember]
	ID        column.IDColumn[projectdomain.ProjectMember, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID column.Column[projectdomain.ProjectMember, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	UserID    column.Column[projectdomain.ProjectMember, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role      column.Column[projectdomain.ProjectMember, string]                     `dbx:"role,index"`
	CreatedAt column.Column[projectdomain.ProjectMember, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[projectdomain.ProjectMember, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMemberSchema = schema.MustSchema("project_members", ProjectMemberSchemaDef{})
