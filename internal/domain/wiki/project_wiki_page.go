// Package wiki defines wiki domain models.
package wiki

import "time"

type ProjectWikiPage struct {
	ID                 int64     `dbx:"id"                     json:"id"`
	ProjectID          int64     `dbx:"project_id"             json:"project_id"`
	Slug               string    `dbx:"slug"                   json:"slug"`
	Title              string    `dbx:"title"                  json:"title"`
	Content            string    `dbx:"content"                json:"content"`
	Format             string    `dbx:"format"                 json:"format"`
	AuthorUserID       int64     `dbx:"author_user_id"         json:"author_user_id"`
	LastEditedByUserID int64     `dbx:"last_edited_by_user_id" json:"last_edited_by_user_id"`
	CreatedAt          time.Time `dbx:"created_at"             json:"created_at"`
	UpdatedAt          time.Time `dbx:"updated_at"             json:"updated_at"`
}
