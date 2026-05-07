package dbschema

import (
	"time"

	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type UserAccessTokenSchemaDef struct {
	schema.Schema[identity.UserAccessToken]
	ID        column.IDColumn[identity.UserAccessToken, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	UserID    column.Column[identity.UserAccessToken, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Name      column.Column[identity.UserAccessToken, string]                     `dbx:"name"`
	Token     column.Column[identity.UserAccessToken, string]                     `dbx:"token,unique"`
	CreatedAt column.Column[identity.UserAccessToken, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[identity.UserAccessToken, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var UserAccessTokenSchema = schema.MustSchema("user_access_tokens", UserAccessTokenSchemaDef{})
