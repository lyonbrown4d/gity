package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
)

type ProjectPackageVersionSchemaDef struct {
	schema.Schema[packagedomain.ProjectPackageVersion]
	ID               column.IDColumn[packagedomain.ProjectPackageVersion, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageID column.Column[packagedomain.ProjectPackageVersion, int64]                      `dbx:"project_package_id,index,ref=project_packages.id,ondelete=cascade"`
	Version          column.Column[packagedomain.ProjectPackageVersion, string]                     `dbx:"version,index"`
	Status           column.Column[packagedomain.ProjectPackageVersion, string]                     `dbx:"status,index"`
	CreatedAt        column.Column[packagedomain.ProjectPackageVersion, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        column.Column[packagedomain.ProjectPackageVersion, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageVersionSchema = schema.MustSchema("project_package_versions", ProjectPackageVersionSchemaDef{})
