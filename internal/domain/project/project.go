package project

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type Project struct {
	ID            int64     `dbx:"id"`
	NamespaceID   int64     `dbx:"namespace_id"`
	Name          string    `dbx:"name"`
	PathKey       string    `dbx:"path_key"`
	FullPath      string    `dbx:"full_path"`
	Visibility    string    `dbx:"visibility"`
	Description   string    `dbx:"description"`
	DefaultBranch string    `dbx:"default_branch"`
	CreatedAt     time.Time `dbx:"created_at"`
	UpdatedAt     time.Time `dbx:"updated_at"`
}

type ProjectSchemaDef struct {
	schema.Schema[Project]
	ID            column.IDColumn[Project, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	NamespaceID   column.Column[Project, int64]                      `dbx:"namespace_id,index,ref=namespaces.id,ondelete=cascade"`
	Name          column.Column[Project, string]                     `dbx:"name"`
	PathKey       column.Column[Project, string]                     `dbx:"path_key"`
	FullPath      column.Column[Project, string]                     `dbx:"full_path,unique"`
	Visibility    column.Column[Project, string]                     `dbx:"visibility,index"`
	Description   column.Column[Project, string]                     `dbx:"description,null"`
	DefaultBranch column.Column[Project, string]                     `dbx:"default_branch"`
	CreatedAt     column.Column[Project, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     column.Column[Project, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectSchema = schema.MustSchema("projects", ProjectSchemaDef{})
