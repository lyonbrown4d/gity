package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
)

type UserSchemaDef struct {
	schema.Schema[identity.User]
	ID           column.IDColumn[identity.User, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Username     column.Column[identity.User, string]                     `dbx:"username,unique"`
	DisplayName  column.Column[identity.User, string]                     `dbx:"display_name"`
	Email        column.Column[identity.User, string]                     `dbx:"email,unique,null"`
	IsSuperAdmin column.Column[identity.User, int]                        `dbx:"is_super_admin"`
	CreatedAt    column.Column[identity.User, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[identity.User, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var UserSchema = schema.MustSchema("users", UserSchemaDef{})
