package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectPackageVersion]
	ID               dbx.IDColumn[ProjectPackageVersion, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageID dbx.Column[ProjectPackageVersion, int64]                    `dbx:"project_package_id,index,ref=project_packages.id,ondelete=cascade"`
	Version          dbx.Column[ProjectPackageVersion, string]                   `dbx:"version,index"`
	Status           dbx.Column[ProjectPackageVersion, string]                   `dbx:"status,index"`
	CreatedAt        dbx.Column[ProjectPackageVersion, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        dbx.Column[ProjectPackageVersion, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageVersionSchema = dbx.MustSchema("project_package_versions", ProjectPackageVersionSchemaDef{})
