package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
)

type Project struct {
	ID            int64     `dbx:"id"`
	NamespaceID   int64     `dbx:"namespace_id"`
	Name          string    `dbx:"name"`
	PathKey       string    `dbx:"path_key"`
	FullPath      string    `dbx:"full_path"`
	Description   string    `dbx:"description"`
	DefaultBranch string    `dbx:"default_branch"`
	CreatedAt     time.Time `dbx:"created_at"`
	UpdatedAt     time.Time `dbx:"updated_at"`
}

type ProjectSchemaDef struct {
	dbx.Schema[Project]
	ID            dbx.IDColumn[Project, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	NamespaceID   dbx.Column[Project, int64]                    `dbx:"namespace_id,index,ref=namespaces.id,ondelete=cascade"`
	Name          dbx.Column[Project, string]                   `dbx:"name"`
	PathKey       dbx.Column[Project, string]                   `dbx:"path_key"`
	FullPath      dbx.Column[Project, string]                   `dbx:"full_path,unique"`
	Description   dbx.Column[Project, string]                   `dbx:"description,null"`
	DefaultBranch dbx.Column[Project, string]                   `dbx:"default_branch"`
	CreatedAt     dbx.Column[Project, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     dbx.Column[Project, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectSchema = dbx.MustSchema("projects", ProjectSchemaDef{})
