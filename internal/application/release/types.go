package release

import (
	"log/slog"
	"time"

	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
)

type Service struct {
	logger        *slog.Logger
	projectRepo   gitports.ProjectRepository
	releaseRepo   gitports.ProjectReleaseRepository
	linkRepo      gitports.ProjectReleaseLinkRepository
	gitRepository gitports.GitRepository
	gitRunner     gitports.GitRunner
}

type CreateReleaseInput struct {
	TagName         string     `json:"tag_name"`
	Name            string     `json:"name"`
	Description     string     `json:"description"`
	SourceRef       string     `json:"source_ref"`
	CreateTag       bool       `json:"create_tag"`
	ReleasedAt      *time.Time `json:"released_at"`
	CreatedByUserID int64      `json:"created_by_user_id"`
}

type UpdateReleaseInput struct {
	Name        *string    `json:"name"`
	Description *string    `json:"description"`
	ReleasedAt  *time.Time `json:"released_at"`
}

type CreateReleaseLinkInput struct {
	Name     string `json:"name"`
	URL      string `json:"url"`
	LinkType string `json:"link_type"`
}

type CreateTagInput struct {
	Name      string `json:"name"`
	SourceRef string `json:"source_ref"`
}

type ReleaseDetail struct {
	Release releasedomain.ProjectRelease       `json:"release"`
	Links   []releasedomain.ProjectReleaseLink `json:"links"`
	Tag     *gitports.Tag                      `json:"tag,omitempty"`
}
