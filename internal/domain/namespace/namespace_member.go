package namespace

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
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
	schema.Schema[NamespaceMember]
	ID          column.IDColumn[NamespaceMember, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	NamespaceID column.Column[NamespaceMember, int64]                      `dbx:"namespace_id,index,ref=namespaces.id,ondelete=cascade"`
	UserID      column.Column[NamespaceMember, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role        column.Column[NamespaceMember, string]                     `dbx:"role,index"`
	CreatedAt   column.Column[NamespaceMember, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[NamespaceMember, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceMemberSchema = schema.MustSchema("namespace_members", NamespaceMemberSchemaDef{})
