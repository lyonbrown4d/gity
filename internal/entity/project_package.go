package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
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
	dbx.Schema[ProjectPackage]
	ID        dbx.IDColumn[ProjectPackage, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID dbx.Column[ProjectPackage, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Type      dbx.Column[ProjectPackage, string]                   `dbx:"type,index"`
	Name      dbx.Column[ProjectPackage, string]                   `dbx:"name,index"`
	CreatedAt dbx.Column[ProjectPackage, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt dbx.Column[ProjectPackage, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageSchema = dbx.MustSchema("project_packages", ProjectPackageSchemaDef{})
