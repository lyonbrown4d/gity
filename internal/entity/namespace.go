package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type Namespace struct {
	ID          int64     `dbx:"id"`
	Kind        string    `dbx:"kind"`
	Name        string    `dbx:"name"`
	PathKey     string    `dbx:"path_key"`
	FullPath    string    `dbx:"full_path"`
	Description string    `dbx:"description"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}

type NamespaceSchemaDef struct {
	schema.Schema[Namespace]
	ID          column.IDColumn[Namespace, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Kind        column.Column[Namespace, string]                     `dbx:"kind,index"`
	Name        column.Column[Namespace, string]                     `dbx:"name"`
	PathKey     column.Column[Namespace, string]                     `dbx:"path_key,unique"`
	FullPath    column.Column[Namespace, string]                     `dbx:"full_path,unique"`
	Description column.Column[Namespace, string]                     `dbx:"description,null"`
	CreatedAt   column.Column[Namespace, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[Namespace, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceSchema = schema.MustSchema("namespaces", NamespaceSchemaDef{})
