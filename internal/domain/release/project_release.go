// Package release defines project release domain models.
package release

import "time"

type ProjectRelease struct {
	ID              int64     `dbx:"id"               json:"id"`
	ProjectID       int64     `dbx:"project_id"       json:"project_id"`
	TagName         string    `dbx:"tag_name"         json:"tag_name"`
	Name            string    `dbx:"name"             json:"name"`
	Description     string    `dbx:"description"      json:"description"`
	CreatedByUserID int64     `dbx:"created_by_user_id" json:"created_by_user_id"`
	ReleasedAt      time.Time `dbx:"released_at"      json:"released_at"`
	CreatedAt       time.Time `dbx:"created_at"       json:"created_at"`
	UpdatedAt       time.Time `dbx:"updated_at"       json:"updated_at"`
}

type ProjectReleaseLink struct {
	ID               int64     `dbx:"id"                 json:"id"`
	ProjectReleaseID int64     `dbx:"project_release_id" json:"project_release_id"`
	Name             string    `dbx:"name"               json:"name"`
	URL              string    `dbx:"url"                json:"url"`
	LinkType         string    `dbx:"link_type"          json:"link_type"`
	CreatedAt        time.Time `dbx:"created_at"         json:"created_at"`
	UpdatedAt        time.Time `dbx:"updated_at"         json:"updated_at"`
}
