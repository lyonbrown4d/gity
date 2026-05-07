package identity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type User struct {
	ID          int64     `dbx:"id"`
	Username    string    `dbx:"username"`
	DisplayName string    `dbx:"display_name"`
	Email       string    `dbx:"email"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}

type UserSchemaDef struct {
	schema.Schema[User]
	ID          column.IDColumn[User, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Username    column.Column[User, string]                     `dbx:"username,unique"`
	DisplayName column.Column[User, string]                     `dbx:"display_name"`
	Email       column.Column[User, string]                     `dbx:"email,unique,null"`
	CreatedAt   column.Column[User, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[User, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var UserSchema = schema.MustSchema("users", UserSchemaDef{})
