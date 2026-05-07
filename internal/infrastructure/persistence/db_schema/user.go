package dbschema

import (
	"time"

	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type UserSchemaDef struct {
	schema.Schema[identity.User]
	ID          column.IDColumn[identity.User, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Username    column.Column[identity.User, string]                     `dbx:"username,unique"`
	DisplayName column.Column[identity.User, string]                     `dbx:"display_name"`
	Email       column.Column[identity.User, string]                     `dbx:"email,unique,null"`
	CreatedAt   column.Column[identity.User, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[identity.User, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var UserSchema = schema.MustSchema("users", UserSchemaDef{})
