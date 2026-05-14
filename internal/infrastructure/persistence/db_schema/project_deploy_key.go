package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
)

type ProjectDeployKeySchemaDef struct {
	schema.Schema[identity.ProjectDeployKey]
	ID              column.IDColumn[identity.ProjectDeployKey, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID       column.Column[identity.ProjectDeployKey, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Title           column.Column[identity.ProjectDeployKey, string]                     `dbx:"title"`
	Fingerprint     column.Column[identity.ProjectDeployKey, string]                     `dbx:"fingerprint,index"`
	PublicKey       column.Column[identity.ProjectDeployKey, string]                     `dbx:"public_key"`
	CanPush         column.Column[identity.ProjectDeployKey, int]                        `dbx:"can_push"`
	CreatedByUserID column.Column[identity.ProjectDeployKey, int64]                      `dbx:"created_by_user_id,index,ref=users.id"`
	LastUsedAt      column.Column[identity.ProjectDeployKey, time.Time]                  `dbx:"last_used_at,type=TIMESTAMP,null"`
	CreatedAt       column.Column[identity.ProjectDeployKey, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[identity.ProjectDeployKey, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectDeployKeySchema = schema.MustSchema("project_deploy_keys", ProjectDeployKeySchemaDef{})
