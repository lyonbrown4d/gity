package ci

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectRunner struct {
	ID            int64     `dbx:"id" json:"id"`
	ProjectID     int64     `dbx:"project_id" json:"project_id"`
	Name          string    `dbx:"name" json:"name"`
	Description   string    `dbx:"description" json:"description"`
	Tags          string    `dbx:"tags" json:"tags"`
	Token         string    `dbx:"token" json:"token"`
	Status        string    `dbx:"status" json:"status"`
	Active        int       `dbx:"active" json:"active"`
	LastContactAt time.Time `dbx:"last_contact_at" json:"last_contact_at"`
	CreatedAt     time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt     time.Time `dbx:"updated_at" json:"updated_at"`
}

type ProjectRunnerSchemaDef struct {
	schema.Schema[ProjectRunner]
	ID            column.IDColumn[ProjectRunner, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID     column.Column[ProjectRunner, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Name          column.Column[ProjectRunner, string]                     `dbx:"name,index"`
	Description   column.Column[ProjectRunner, string]                     `dbx:"description,type=TEXT,null"`
	Tags          column.Column[ProjectRunner, string]                     `dbx:"tags,index,null"`
	Token         column.Column[ProjectRunner, string]                     `dbx:"token,unique"`
	Status        column.Column[ProjectRunner, string]                     `dbx:"status,index"`
	Active        column.Column[ProjectRunner, int]                        `dbx:"active,index"`
	LastContactAt column.Column[ProjectRunner, time.Time]                  `dbx:"last_contact_at,type=TIMESTAMP,index"`
	CreatedAt     column.Column[ProjectRunner, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     column.Column[ProjectRunner, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectRunnerSchema = schema.MustSchema("project_runners", ProjectRunnerSchemaDef{})
