package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
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
	dbx.Schema[ProjectLFSObject]
	ID         dbx.IDColumn[ProjectLFSObject, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID  dbx.Column[ProjectLFSObject, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OID        dbx.Column[ProjectLFSObject, string]                   `dbx:"oid,index"`
	ByteSize   dbx.Column[ProjectLFSObject, int64]                    `dbx:"byte_size"`
	StorageKey dbx.Column[ProjectLFSObject, string]                   `dbx:"storage_key,unique"`
	CreatedAt  dbx.Column[ProjectLFSObject, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt  dbx.Column[ProjectLFSObject, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSObjectSchema = dbx.MustSchema("project_lfs_objects", ProjectLFSObjectSchemaDef{})
