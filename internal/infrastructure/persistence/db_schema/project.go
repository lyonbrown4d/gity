package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
)

type ProjectSchemaDef struct {
	schema.Schema[projectdomain.Project]
	ID             column.IDColumn[projectdomain.Project, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	OrganizationID column.Column[projectdomain.Project, int64]                      `dbx:"organization_id,index,ref=organizations.id,ondelete=cascade"`
	Name           column.Column[projectdomain.Project, string]                     `dbx:"name"`
	PathKey        column.Column[projectdomain.Project, string]                     `dbx:"path_key"`
	FullPath       column.Column[projectdomain.Project, string]                     `dbx:"full_path,unique"`
	Visibility     column.Column[projectdomain.Project, string]                     `dbx:"visibility,index"`
	Description    column.Column[projectdomain.Project, string]                     `dbx:"description,null"`
	DefaultBranch  column.Column[projectdomain.Project, string]                     `dbx:"default_branch"`
	Status         column.Column[projectdomain.Project, string]                     `dbx:"status,index"`
	DeletedAt      column.Column[projectdomain.Project, time.Time]                  `dbx:"deleted_at,type=TIMESTAMP,null"`
	CreatedAt      column.Column[projectdomain.Project, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[projectdomain.Project, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectSchema = schema.MustSchema("projects", ProjectSchemaDef{})
