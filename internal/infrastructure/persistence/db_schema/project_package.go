package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
)

type ProjectPackageSchemaDef struct {
	schema.Schema[packagedomain.ProjectPackage]
	ID        column.IDColumn[packagedomain.ProjectPackage, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID column.Column[packagedomain.ProjectPackage, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Type      column.Column[packagedomain.ProjectPackage, string]                     `dbx:"type,index"`
	Name      column.Column[packagedomain.ProjectPackage, string]                     `dbx:"name,index"`
	CreatedAt column.Column[packagedomain.ProjectPackage, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[packagedomain.ProjectPackage, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageSchema = schema.MustSchema("project_packages", ProjectPackageSchemaDef{})
