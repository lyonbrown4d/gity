package dbschema

import (
	"time"

	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type NamespaceSchemaDef struct {
	schema.Schema[namespacedomain.Namespace]
	ID          column.IDColumn[namespacedomain.Namespace, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Kind        column.Column[namespacedomain.Namespace, string]                     `dbx:"kind,index"`
	Name        column.Column[namespacedomain.Namespace, string]                     `dbx:"name"`
	PathKey     column.Column[namespacedomain.Namespace, string]                     `dbx:"path_key,unique"`
	FullPath    column.Column[namespacedomain.Namespace, string]                     `dbx:"full_path,unique"`
	Description column.Column[namespacedomain.Namespace, string]                     `dbx:"description,null"`
	CreatedAt   column.Column[namespacedomain.Namespace, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[namespacedomain.Namespace, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceSchema = schema.MustSchema("namespaces", NamespaceSchemaDef{})
