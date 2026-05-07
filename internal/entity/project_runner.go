package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectRunner]
	ID            dbx.IDColumn[ProjectRunner, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID     dbx.Column[ProjectRunner, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Name          dbx.Column[ProjectRunner, string]                   `dbx:"name,index"`
	Description   dbx.Column[ProjectRunner, string]                   `dbx:"description,type=TEXT,null"`
	Tags          dbx.Column[ProjectRunner, string]                   `dbx:"tags,index,null"`
	Token         dbx.Column[ProjectRunner, string]                   `dbx:"token,unique"`
	Status        dbx.Column[ProjectRunner, string]                   `dbx:"status,index"`
	Active        dbx.Column[ProjectRunner, int]                      `dbx:"active,index"`
	LastContactAt dbx.Column[ProjectRunner, time.Time]                `dbx:"last_contact_at,type=TIMESTAMP,index"`
	CreatedAt     dbx.Column[ProjectRunner, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     dbx.Column[ProjectRunner, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectRunnerSchema = dbx.MustSchema("project_runners", ProjectRunnerSchemaDef{})
