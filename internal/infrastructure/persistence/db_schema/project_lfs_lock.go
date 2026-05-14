package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	lfsdomain "github.com/lyonbrown4d/gity/internal/domain/lfs"
)

type ProjectLFSLockSchemaDef struct {
	schema.Schema[lfsdomain.ProjectLFSLock]
	ID          column.IDColumn[lfsdomain.ProjectLFSLock, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID   column.Column[lfsdomain.ProjectLFSLock, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OwnerUserID column.Column[lfsdomain.ProjectLFSLock, int64]                      `dbx:"owner_user_id,index,ref=users.id,ondelete=cascade"`
	Path        column.Column[lfsdomain.ProjectLFSLock, string]                     `dbx:"path,index"`
	CreatedAt   column.Column[lfsdomain.ProjectLFSLock, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[lfsdomain.ProjectLFSLock, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectLFSLockSchema = schema.MustSchema("project_lfs_locks", ProjectLFSLockSchemaDef{})
