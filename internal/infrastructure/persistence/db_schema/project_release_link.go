package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
)

type ProjectReleaseLinkSchemaDef struct {
	schema.Schema[releasedomain.ProjectReleaseLink]
	ID               column.IDColumn[releasedomain.ProjectReleaseLink, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectReleaseID column.Column[releasedomain.ProjectReleaseLink, int64]                      `dbx:"project_release_id,index,ref=project_releases.id,ondelete=cascade"`
	Name             column.Column[releasedomain.ProjectReleaseLink, string]                     `dbx:"name"`
	URL              column.Column[releasedomain.ProjectReleaseLink, string]                     `dbx:"url"`
	LinkType         column.Column[releasedomain.ProjectReleaseLink, string]                     `dbx:"link_type,index"`
	CreatedAt        column.Column[releasedomain.ProjectReleaseLink, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt        column.Column[releasedomain.ProjectReleaseLink, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectReleaseLinkSchema = schema.MustSchema("project_release_links", ProjectReleaseLinkSchemaDef{})
