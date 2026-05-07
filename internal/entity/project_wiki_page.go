package entity

import (
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
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
	dbx.Schema[ProjectWikiPage]
	ID                 dbx.IDColumn[ProjectWikiPage, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID          dbx.Column[ProjectWikiPage, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	Slug               dbx.Column[ProjectWikiPage, string]                   `dbx:"slug,index"`
	Title              dbx.Column[ProjectWikiPage, string]                   `dbx:"title"`
	Content            dbx.Column[ProjectWikiPage, string]                   `dbx:"content,type=TEXT,null"`
	Format             dbx.Column[ProjectWikiPage, string]                   `dbx:"format,index"`
	AuthorUserID       dbx.Column[ProjectWikiPage, int64]                    `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	LastEditedByUserID dbx.Column[ProjectWikiPage, int64]                    `dbx:"last_edited_by_user_id,index,ref=users.id,ondelete=restrict"`
	CreatedAt          dbx.Column[ProjectWikiPage, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt          dbx.Column[ProjectWikiPage, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
	ProjectSlugUnique  dbx.Unique[ProjectWikiPage]                           `idx:"columns=project_id,slug"`
}

var ProjectWikiPageSchema = dbx.MustSchema("project_wiki_pages", ProjectWikiPageSchemaDef{})
