package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectWikiPage struct {
	ID                 int64     `dbx:"id" json:"id"`
	ProjectID          int64     `dbx:"project_id" json:"project_id"`
	Slug               string    `dbx:"slug" json:"slug"`
	Title              string    `dbx:"title" json:"title"`
	Content            string    `dbx:"content" json:"content"`
	Format             string    `dbx:"format" json:"format"`
	AuthorUserID       int64     `dbx:"author_user_id" json:"author_user_id"`
	LastEditedByUserID int64     `dbx:"last_edited_by_user_id" json:"last_edited_by_user_id"`
	CreatedAt          time.Time `dbx:"created_at" json:"created_at"`
	UpdatedAt          time.Time `dbx:"updated_at" json:"updated_at"`
}

type ProjectWikiPageSchemaDef struct {
	schema.Schema[ProjectWikiPage]
	ID                 column.IDColumn[ProjectWikiPage, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID          column.Column[ProjectWikiPage, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Slug               column.Column[ProjectWikiPage, string]                     `dbx:"slug,index"`
	Title              column.Column[ProjectWikiPage, string]                     `dbx:"title"`
	Content            column.Column[ProjectWikiPage, string]                     `dbx:"content,type=TEXT,null"`
	Format             column.Column[ProjectWikiPage, string]                     `dbx:"format,index"`
	AuthorUserID       column.Column[ProjectWikiPage, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	LastEditedByUserID column.Column[ProjectWikiPage, int64]                      `dbx:"last_edited_by_user_id,index,ref=users.id,ondelete=restrict"`
	CreatedAt          column.Column[ProjectWikiPage, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt          column.Column[ProjectWikiPage, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
	ProjectSlugUnique  schema.Unique[ProjectWikiPage]                             `idx:"columns=project_id,slug"`
}

var ProjectWikiPageSchema = schema.MustSchema("project_wiki_pages", ProjectWikiPageSchemaDef{})
