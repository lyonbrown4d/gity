package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
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
	dbx.Schema[Namespace]
	ID          dbx.IDColumn[Namespace, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	Kind        dbx.Column[Namespace, string]                   `dbx:"kind,index"`
	Name        dbx.Column[Namespace, string]                   `dbx:"name"`
	PathKey     dbx.Column[Namespace, string]                   `dbx:"path_key,unique"`
	FullPath    dbx.Column[Namespace, string]                   `dbx:"full_path,unique"`
	Description dbx.Column[Namespace, string]                   `dbx:"description,null"`
	CreatedAt   dbx.Column[Namespace, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   dbx.Column[Namespace, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var NamespaceSchema = dbx.MustSchema("namespaces", NamespaceSchemaDef{})
