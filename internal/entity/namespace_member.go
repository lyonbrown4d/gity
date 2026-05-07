package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
)

type NamespaceMember struct {
	ID          int64     `dbx:"id"`
	NamespaceID int64     `dbx:"namespace_id"`
	UserID      int64     `dbx:"user_id"`
	Role        string    `dbx:"role"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}

type NamespaceMemberSchemaDef struct {
	dbx.Schema[NamespaceMember]
	ID          dbx.IDColumn[NamespaceMember, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	NamespaceID dbx.Column[NamespaceMember, int64]                    `dbx:"namespace_id,index,ref=namespaces.id,ondelete=cascade"`
	UserID      dbx.Column[NamespaceMember, int64]                    `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role        dbx.Column[NamespaceMember, string]                   `dbx:"role,index"`
	CreatedAt   dbx.Column[NamespaceMember, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   dbx.Column[NamespaceMember, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceMemberSchema = dbx.MustSchema("namespace_members", NamespaceMemberSchemaDef{})
