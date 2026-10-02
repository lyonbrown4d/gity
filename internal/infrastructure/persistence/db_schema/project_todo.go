package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
)

type ProjectTodoSchemaDef struct {
	schema.Schema[tododomain.ProjectTodo]
	ID             column.IDColumn[tododomain.ProjectTodo, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	UserID         column.Column[tododomain.ProjectTodo, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	OrganizationID column.Column[tododomain.ProjectTodo, int64]                      `dbx:"organization_id,index,ref=organizations.id,ondelete=cascade"`
	ProjectID      column.Column[tododomain.ProjectTodo, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Kind           column.Column[tododomain.ProjectTodo, string]                     `dbx:"kind,index"`
	State          column.Column[tododomain.ProjectTodo, string]                     `dbx:"state,index"`
	TargetType     column.Column[tododomain.ProjectTodo, string]                     `dbx:"target_type,index"`
	TargetID       column.Column[tododomain.ProjectTodo, string]                     `dbx:"target_id,index"`
	Title          column.Column[tododomain.ProjectTodo, string]                     `dbx:"title"`
	Summary        column.Column[tododomain.ProjectTodo, string]                     `dbx:"summary,type=TEXT"`
	ActionURL      column.Column[tododomain.ProjectTodo, string]                     `dbx:"action_url"`
	CreatedAt      column.Column[tododomain.ProjectTodo, time.Time]                  `dbx:"created_at,type=TIMESTAMP,index"`
	UpdatedAt      column.Column[tododomain.ProjectTodo, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	DoneAt         column.Column[tododomain.ProjectTodo, time.Time]                  `dbx:"done_at,type=TIMESTAMP,null"`
}

var ProjectTodoSchema = schema.MustSchema("project_todos", ProjectTodoSchemaDef{})
