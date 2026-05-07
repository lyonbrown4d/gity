package packageregistry

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPackageFile struct {
	ID                      int64     `dbx:"id"`
	ProjectPackageVersionID int64     `dbx:"project_package_version_id"`
	FileName                string    `dbx:"file_name"`
	FilePath                string    `dbx:"file_path"`
	ContentType             string    `dbx:"content_type"`
	ByteSize                int64     `dbx:"byte_size"`
	StorageKey              string    `dbx:"storage_key"`
	CreatedAt               time.Time `dbx:"created_at"`
	UpdatedAt               time.Time `dbx:"updated_at"`
}

type ProjectPackageFileSchemaDef struct {
	schema.Schema[ProjectPackageFile]
	ID                      column.IDColumn[ProjectPackageFile, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageVersionID column.Column[ProjectPackageFile, int64]                      `dbx:"project_package_version_id,index,ref=project_package_versions.id,ondelete=cascade"`
	FileName                column.Column[ProjectPackageFile, string]                     `dbx:"file_name"`
	FilePath                column.Column[ProjectPackageFile, string]                     `dbx:"file_path,index"`
	ContentType             column.Column[ProjectPackageFile, string]                     `dbx:"content_type,null"`
	ByteSize                column.Column[ProjectPackageFile, int64]                      `dbx:"byte_size"`
	StorageKey              column.Column[ProjectPackageFile, string]                     `dbx:"storage_key,unique"`
	CreatedAt               column.Column[ProjectPackageFile, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt               column.Column[ProjectPackageFile, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageFileSchema = schema.MustSchema("project_package_files", ProjectPackageFileSchemaDef{})
