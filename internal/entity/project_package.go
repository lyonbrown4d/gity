package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPackage struct {
	ID        int64     `dbx:"id"`
	ProjectID int64     `dbx:"project_id"`
	Type      string    `dbx:"type"`
	Name      string    `dbx:"name"`
	CreatedAt time.Time `dbx:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at"`
}

type ProjectPackageSchemaDef struct {
	schema.Schema[ProjectPackage]
	ID        column.IDColumn[ProjectPackage, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID column.Column[ProjectPackage, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Type      column.Column[ProjectPackage, string]                     `dbx:"type,index"`
	Name      column.Column[ProjectPackage, string]                     `dbx:"name,index"`
	CreatedAt column.Column[ProjectPackage, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[ProjectPackage, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageSchema = schema.MustSchema("project_packages", ProjectPackageSchemaDef{})
