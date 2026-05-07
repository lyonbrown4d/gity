package lfs

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectLFSLock struct {
	ID          int64     `dbx:"id"`
	ProjectID   int64     `dbx:"project_id"`
	OwnerUserID int64     `dbx:"owner_user_id"`
	Path        string    `dbx:"path"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}

type ProjectLFSLockSchemaDef struct {
	schema.Schema[ProjectLFSLock]
	ID          column.IDColumn[ProjectLFSLock, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID   column.Column[ProjectLFSLock, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OwnerUserID column.Column[ProjectLFSLock, int64]                      `dbx:"owner_user_id,index,ref=users.id,ondelete=cascade"`
	Path        column.Column[ProjectLFSLock, string]                     `dbx:"path,index"`
	CreatedAt   column.Column[ProjectLFSLock, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[ProjectLFSLock, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSLockSchema = schema.MustSchema("project_lfs_locks", ProjectLFSLockSchemaDef{})
