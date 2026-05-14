package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
)

type ProjectAccessTokenSchemaDef struct {
	schema.Schema[identity.ProjectAccessToken]
	ID              column.IDColumn[identity.ProjectAccessToken, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID       column.Column[identity.ProjectAccessToken, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Kind            column.Column[identity.ProjectAccessToken, string]                     `dbx:"kind,index"`
	Name            column.Column[identity.ProjectAccessToken, string]                     `dbx:"name"`
	Username        column.Column[identity.ProjectAccessToken, string]                     `dbx:"username,index"`
	Token           column.Column[identity.ProjectAccessToken, string]                     `dbx:"token,unique"`
	Scopes          column.Column[identity.ProjectAccessToken, string]                     `dbx:"scopes"`
	CreatedByUserID column.Column[identity.ProjectAccessToken, int64]                      `dbx:"created_by_user_id,index,ref=users.id"`
	ExpiresAt       column.Column[identity.ProjectAccessToken, time.Time]                  `dbx:"expires_at,type=TIMESTAMP,null"`
	RevokedAt       column.Column[identity.ProjectAccessToken, time.Time]                  `dbx:"revoked_at,type=TIMESTAMP,null"`
	LastUsedAt      column.Column[identity.ProjectAccessToken, time.Time]                  `dbx:"last_used_at,type=TIMESTAMP,null"`
	CreatedAt       column.Column[identity.ProjectAccessToken, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[identity.ProjectAccessToken, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectAccessTokenSchema = schema.MustSchema("project_access_tokens", ProjectAccessTokenSchemaDef{})
