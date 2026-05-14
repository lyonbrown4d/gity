package release

import gitports "github.com/lyonbrown4d/gity/internal/application/ports"

type projectReleaseInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type projectTagInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type createTagInput struct {
	ProjectID     int64         `path:"id"`
	Authorization string        `header:"Authorization"`
	Body          createTagBody `json:"body"`
}

type deleteTagInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
	Name          string `query:"name"`
}

type createReleaseInput struct {
	ProjectID     int64             `path:"id"`
	Authorization string            `header:"Authorization"`
	Body          createReleaseBody `json:"body"`
}

type releaseByIDInput struct {
	ProjectID     int64             `path:"id"`
	ReleaseID     int64             `path:"release_id"`
	Authorization string            `header:"Authorization"`
	DeleteTag     bool              `query:"delete_tag"`
	Body          updateReleaseBody `json:"body"`
}

type createReleaseLinkInput struct {
	ProjectID     int64                 `path:"id"`
	ReleaseID     int64                 `path:"release_id"`
	Authorization string                `header:"Authorization"`
	Body          createReleaseLinkBody `json:"body"`
}

type releaseLinkInput struct {
	ProjectID     int64  `path:"id"`
	ReleaseID     int64  `path:"release_id"`
	LinkID        int64  `path:"link_id"`
	Authorization string `header:"Authorization"`
}

type releaseOutput struct {
	Body any `json:"body"`
}

type createTagBody struct {
	Name      string `json:"name"`
	SourceRef string `json:"source_ref"`
}

type createReleaseBody struct {
	TagName         string `json:"tag_name"`
	Name            string `json:"name"`
	Description     string `json:"description"`
	SourceRef       string `json:"source_ref"`
	CreateTag       bool   `json:"create_tag"`
	ReleasedAt      string `json:"released_at"`
	CreatedByUserID int64  `json:"created_by_user_id"`
}

type updateReleaseBody struct {
	Name        *string `json:"name"`
	Description *string `json:"description"`
	ReleasedAt  string  `json:"released_at"`
}

type createReleaseLinkBody struct {
	Name     string `json:"name"`
	URL      string `json:"url"`
	LinkType string `json:"link_type"`
}

type releaseDetailView struct {
	Release releaseView       `json:"release"`
	Links   []releaseLinkView `json:"links"`
	Tag     *gitports.Tag     `json:"tag,omitempty"`
}

type releaseView struct {
	ID              string `json:"id"`
	ProjectID       string `json:"project_id"`
	TagName         string `json:"tag_name"`
	Name            string `json:"name"`
	Description     string `json:"description"`
	CreatedByUserID string `json:"created_by_user_id"`
	ReleasedAt      string `json:"released_at"`
	CreatedAt       string `json:"created_at"`
	UpdatedAt       string `json:"updated_at"`
}

type releaseLinkView struct {
	ID               string `json:"id"`
	ProjectReleaseID string `json:"project_release_id"`
	Name             string `json:"name"`
	URL              string `json:"url"`
	LinkType         string `json:"link_type"`
	CreatedAt        string `json:"created_at"`
	UpdatedAt        string `json:"updated_at"`
}

func (in projectReleaseInput) AuthorizationHeader() string { return in.Authorization }
func (in projectReleaseInput) ProjectIDValue() int64       { return in.ProjectID }
func (in projectTagInput) AuthorizationHeader() string     { return in.Authorization }
func (in projectTagInput) ProjectIDValue() int64           { return in.ProjectID }
func (in createTagInput) AuthorizationHeader() string      { return in.Authorization }
func (in createTagInput) ProjectIDValue() int64            { return in.ProjectID }
func (in deleteTagInput) AuthorizationHeader() string      { return in.Authorization }
func (in deleteTagInput) ProjectIDValue() int64            { return in.ProjectID }
func (in createReleaseInput) AuthorizationHeader() string  { return in.Authorization }
func (in createReleaseInput) ProjectIDValue() int64        { return in.ProjectID }
func (in releaseByIDInput) AuthorizationHeader() string    { return in.Authorization }
func (in releaseByIDInput) ProjectIDValue() int64          { return in.ProjectID }
func (in createReleaseLinkInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in createReleaseLinkInput) ProjectIDValue() int64 { return in.ProjectID }
func (in releaseLinkInput) AuthorizationHeader() string { return in.Authorization }
func (in releaseLinkInput) ProjectIDValue() int64       { return in.ProjectID }
