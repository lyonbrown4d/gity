package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectLFSObject struct {
	ID         int64     `dbx:"id"`
	ProjectID  int64     `dbx:"project_id"`
	OID        string    `dbx:"oid"`
	ByteSize   int64     `dbx:"byte_size"`
	StorageKey string    `dbx:"storage_key"`
	CreatedAt  time.Time `dbx:"created_at"`
	UpdatedAt  time.Time `dbx:"updated_at"`
}

type ProjectLFSObjectSchemaDef struct {
	schema.Schema[ProjectLFSObject]
	ID         column.IDColumn[ProjectLFSObject, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID  column.Column[ProjectLFSObject, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OID        column.Column[ProjectLFSObject, string]                     `dbx:"oid,index"`
	ByteSize   column.Column[ProjectLFSObject, int64]                      `dbx:"byte_size"`
	StorageKey column.Column[ProjectLFSObject, string]                     `dbx:"storage_key,unique"`
	CreatedAt  column.Column[ProjectLFSObject, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt  column.Column[ProjectLFSObject, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSObjectSchema = schema.MustSchema("project_lfs_objects", ProjectLFSObjectSchemaDef{})
