package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[UserAccessToken]
	ID        dbx.IDColumn[UserAccessToken, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	UserID    dbx.Column[UserAccessToken, int64]                    `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Name      dbx.Column[UserAccessToken, string]                   `dbx:"name"`
	Token     dbx.Column[UserAccessToken, string]                   `dbx:"token,unique"`
	CreatedAt dbx.Column[UserAccessToken, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt dbx.Column[UserAccessToken, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var UserAccessTokenSchema = dbx.MustSchema("user_access_tokens", UserAccessTokenSchemaDef{})
