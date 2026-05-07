package dbschema

import (
	"time"

	packagedomain "github.com/DaiYuANg/gity/internal/domain/package_registry"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPackageFileSchemaDef struct {
	schema.Schema[packagedomain.ProjectPackageFile]
	ID                      column.IDColumn[packagedomain.ProjectPackageFile, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectPackageVersionID column.Column[packagedomain.ProjectPackageFile, int64]                      `dbx:"project_package_version_id,index,ref=project_package_versions.id,ondelete=cascade"`
	FileName                column.Column[packagedomain.ProjectPackageFile, string]                     `dbx:"file_name"`
	FilePath                column.Column[packagedomain.ProjectPackageFile, string]                     `dbx:"file_path,index"`
	ContentType             column.Column[packagedomain.ProjectPackageFile, string]                     `dbx:"content_type,null"`
	ByteSize                column.Column[packagedomain.ProjectPackageFile, int64]                      `dbx:"byte_size"`
	StorageKey              column.Column[packagedomain.ProjectPackageFile, string]                     `dbx:"storage_key,unique"`
	CreatedAt               column.Column[packagedomain.ProjectPackageFile, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt               column.Column[packagedomain.ProjectPackageFile, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectPackageFileSchema = schema.MustSchema("project_package_files", ProjectPackageFileSchemaDef{})
