package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[User]
	ID          dbx.IDColumn[User, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	Username    dbx.Column[User, string]                   `dbx:"username,unique"`
	DisplayName dbx.Column[User, string]                   `dbx:"display_name"`
	Email       dbx.Column[User, string]                   `dbx:"email,unique,null"`
	CreatedAt   dbx.Column[User, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   dbx.Column[User, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var UserSchema = dbx.MustSchema("users", UserSchemaDef{})
