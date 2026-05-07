package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type UserAccessToken struct {
	ID        int64     `dbx:"id"`
	UserID    int64     `dbx:"user_id"`
	Name      string    `dbx:"name"`
	Token     string    `dbx:"token"`
	CreatedAt time.Time `dbx:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at"`
}

type UserAccessTokenSchemaDef struct {
	schema.Schema[UserAccessToken]
	ID        column.IDColumn[UserAccessToken, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	UserID    column.Column[UserAccessToken, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Name      column.Column[UserAccessToken, string]                     `dbx:"name"`
	Token     column.Column[UserAccessToken, string]                     `dbx:"token,unique"`
	CreatedAt column.Column[UserAccessToken, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt column.Column[UserAccessToken, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var UserAccessTokenSchema = schema.MustSchema("user_access_tokens", UserAccessTokenSchemaDef{})
