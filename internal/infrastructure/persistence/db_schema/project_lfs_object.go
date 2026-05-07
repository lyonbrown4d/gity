package dbschema

import (
	"time"

	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectLFSObjectSchemaDef struct {
	schema.Schema[lfsdomain.ProjectLFSObject]
	ID         column.IDColumn[lfsdomain.ProjectLFSObject, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID  column.Column[lfsdomain.ProjectLFSObject, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OID        column.Column[lfsdomain.ProjectLFSObject, string]                     `dbx:"oid,index"`
	ByteSize   column.Column[lfsdomain.ProjectLFSObject, int64]                      `dbx:"byte_size"`
	StorageKey column.Column[lfsdomain.ProjectLFSObject, string]                     `dbx:"storage_key,unique"`
	CreatedAt  column.Column[lfsdomain.ProjectLFSObject, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt  column.Column[lfsdomain.ProjectLFSObject, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSObjectSchema = schema.MustSchema("project_lfs_objects", ProjectLFSObjectSchemaDef{})
