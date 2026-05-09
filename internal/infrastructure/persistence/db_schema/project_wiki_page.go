package dbschema

import (
	"time"

	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectWikiPageSchemaDef struct {
	schema.Schema[wikidomain.ProjectWikiPage]
	ID                 column.IDColumn[wikidomain.ProjectWikiPage, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID          column.Column[wikidomain.ProjectWikiPage, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Slug               column.Column[wikidomain.ProjectWikiPage, string]                     `dbx:"slug,index"`
	Title              column.Column[wikidomain.ProjectWikiPage, string]                     `dbx:"title"`
	Content            column.Column[wikidomain.ProjectWikiPage, string]                     `dbx:"content,type=TEXT,null"`
	Format             column.Column[wikidomain.ProjectWikiPage, string]                     `dbx:"format,index"`
	AuthorUserID       column.Column[wikidomain.ProjectWikiPage, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	LastEditedByUserID column.Column[wikidomain.ProjectWikiPage, int64]                      `dbx:"last_edited_by_user_id,index,ref=users.id,ondelete=restrict"`
	CreatedAt          column.Column[wikidomain.ProjectWikiPage, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt          column.Column[wikidomain.ProjectWikiPage, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	ProjectSlugUnique  schema.Unique[wikidomain.ProjectWikiPage]                             `idx:"columns=project_id|slug"`
}

var ProjectWikiPageSchema = schema.MustSchema("project_wiki_pages", ProjectWikiPageSchemaDef{})
