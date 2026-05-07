package dbschema

import (
	"time"

	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type NamespaceMemberSchemaDef struct {
	schema.Schema[namespacedomain.NamespaceMember]
	ID          column.IDColumn[namespacedomain.NamespaceMember, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	NamespaceID column.Column[namespacedomain.NamespaceMember, int64]                      `dbx:"namespace_id,index,ref=namespaces.id,ondelete=cascade"`
	UserID      column.Column[namespacedomain.NamespaceMember, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role        column.Column[namespacedomain.NamespaceMember, string]                     `dbx:"role,index"`
	CreatedAt   column.Column[namespacedomain.NamespaceMember, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[namespacedomain.NamespaceMember, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceMemberSchema = schema.MustSchema("namespace_members", NamespaceMemberSchemaDef{})
