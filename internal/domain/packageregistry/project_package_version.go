package packageregistry

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPackageVersion struct {
	ID               int64     `dbx:"id"`
	ProjectPackageID int64     `dbx:"project_package_id"`
	Version          string    `dbx:"version"`
	Status           string    `dbx:"status"`
	CreatedAt        time.Time `dbx:"created_at"`
	UpdatedAt        time.Time `dbx:"updated_at"`
}

type ProjectPackageVersionSchemaDef struct {
	schema.Schema[ProjectPackageVersion]
	ID               column.IDColumn[ProjectPackageVersion, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageID column.Column[ProjectPackageVersion, int64]                      `dbx:"project_package_id,index,ref=project_packages.id,ondelete=cascade"`
	Version          column.Column[ProjectPackageVersion, string]                     `dbx:"version,index"`
	Status           column.Column[ProjectPackageVersion, string]                     `dbx:"status,index"`
	CreatedAt        column.Column[ProjectPackageVersion, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        column.Column[ProjectPackageVersion, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageVersionSchema = schema.MustSchema("project_package_versions", ProjectPackageVersionSchemaDef{})
