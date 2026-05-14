package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
)

type ProjectReleaseSchemaDef struct {
	schema.Schema[releasedomain.ProjectRelease]
	ID              column.IDColumn[releasedomain.ProjectRelease, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID       column.Column[releasedomain.ProjectRelease, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	TagName         column.Column[releasedomain.ProjectRelease, string]                     `dbx:"tag_name,index"`
	Name            column.Column[releasedomain.ProjectRelease, string]                     `dbx:"name"`
	Description     column.Column[releasedomain.ProjectRelease, string]                     `dbx:"description,null"`
	CreatedByUserID column.Column[releasedomain.ProjectRelease, int64]                      `dbx:"created_by_user_id,index,ref=users.id"`
	ReleasedAt      column.Column[releasedomain.ProjectRelease, time.Time]                  `dbx:"released_at,type=TIMESTAMP"`
	CreatedAt       column.Column[releasedomain.ProjectRelease, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[releasedomain.ProjectRelease, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectReleaseSchema = schema.MustSchema("project_releases", ProjectReleaseSchemaDef{})
