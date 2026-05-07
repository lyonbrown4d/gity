package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectLFSLock]
	ID          dbx.IDColumn[ProjectLFSLock, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID   dbx.Column[ProjectLFSLock, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OwnerUserID dbx.Column[ProjectLFSLock, int64]                    `dbx:"owner_user_id,index,ref=users.id,ondelete=cascade"`
	Path        dbx.Column[ProjectLFSLock, string]                   `dbx:"path,index"`
	CreatedAt   dbx.Column[ProjectLFSLock, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   dbx.Column[ProjectLFSLock, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSLockSchema = dbx.MustSchema("project_lfs_locks", ProjectLFSLockSchemaDef{})
