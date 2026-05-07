package dbschema

import (
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectPipelineSchemaDef struct {
	schema.Schema[cidomain.ProjectPipeline]
	ID            column.IDColumn[cidomain.ProjectPipeline, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID     column.Column[cidomain.ProjectPipeline, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID           column.Column[cidomain.ProjectPipeline, int64]                      `dbx:"iid,index"`
	Name          column.Column[cidomain.ProjectPipeline, string]                     `dbx:"name"`
	Source        column.Column[cidomain.ProjectPipeline, string]                     `dbx:"source,index"`
	RefName       column.Column[cidomain.ProjectPipeline, string]                     `dbx:"ref_name,index,null"`
	CommitSHA     column.Column[cidomain.ProjectPipeline, string]                     `dbx:"commit_sha,index,null"`
	Status        column.Column[cidomain.ProjectPipeline, string]                     `dbx:"status,index"`
	ConfigSource  column.Column[cidomain.ProjectPipeline, string]                     `dbx:"config_source"`
	ConfigContent column.Column[cidomain.ProjectPipeline, string]                     `dbx:"config_content,type=TEXT"`
	CreatedAt     column.Column[cidomain.ProjectPipeline, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt     column.Column[cidomain.ProjectPipeline, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	StartedAt     column.Column[cidomain.ProjectPipeline, time.Time]                  `dbx:"started_at,type=TIMESTAMP"`
	FinishedAt    column.Column[cidomain.ProjectPipeline, time.Time]                  `dbx:"finished_at,type=TIMESTAMP"`
}

var ProjectPipelineSchema = schema.MustSchema("project_pipelines", ProjectPipelineSchemaDef{})
