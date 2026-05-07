package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectPackageFile]
	ID                      dbx.IDColumn[ProjectPackageFile, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageVersionID dbx.Column[ProjectPackageFile, int64]                    `dbx:"project_package_version_id,index,ref=project_package_versions.id,ondelete=cascade"`
	FileName                dbx.Column[ProjectPackageFile, string]                   `dbx:"file_name"`
	FilePath                dbx.Column[ProjectPackageFile, string]                   `dbx:"file_path,index"`
	ContentType             dbx.Column[ProjectPackageFile, string]                   `dbx:"content_type,null"`
	ByteSize                dbx.Column[ProjectPackageFile, int64]                    `dbx:"byte_size"`
	StorageKey              dbx.Column[ProjectPackageFile, string]                   `dbx:"storage_key,unique"`
	CreatedAt               dbx.Column[ProjectPackageFile, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt               dbx.Column[ProjectPackageFile, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageFileSchema = dbx.MustSchema("project_package_files", ProjectPackageFileSchemaDef{})
