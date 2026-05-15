package release

import (
	"context"
	"errors"
	"net/url"
	"strings"
	"time"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
	"github.com/samber/oops"
)

func (s *Service) loadProject(ctx context.Context, projectID int64) (projectdomain.Project, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, apperror.NotFound("project not found", err)
	}
	return project, nil
}

func (s *Service) loadRelease(ctx context.Context, projectID, releaseID int64) (releasedomain.ProjectRelease, error) {
	item, err := s.releaseRepo.GetByID(ctx, releaseID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return releasedomain.ProjectRelease{}, apperror.NotFound("release not found", err)
		}
		return releasedomain.ProjectRelease{}, oops.In("release").With("project_id", projectID, "release_id", releaseID).Wrapf(err, "load release")
	}
	if item.ProjectID != projectID {
		return releasedomain.ProjectRelease{}, apperror.NotFound("release not found", nil)
	}
	return item, nil
}

func (s *Service) getTag(ctx context.Context, project projectdomain.Project, tagName string) (gitports.Tag, error) {
	tag, ok, err := s.findTag(ctx, project, tagName)
	if err != nil {
		return gitports.Tag{}, err
	}
	if !ok {
		return gitports.Tag{}, apperror.NotFound("git tag not found", gitports.ErrReferenceNotFound)
	}
	return tag, nil
}

func (s *Service) ensureReleaseDoesNotExist(ctx context.Context, projectID int64, tagName string) error {
	_, err := s.releaseRepo.GetByProjectAndTagName(ctx, projectID, tagName)
	if err == nil {
		return apperror.Conflict("release already exists for tag", oops.In("release").With("project_id", projectID, "tag", tagName).New("release already exists for tag"))
	}
	if errors.Is(err, gitports.ErrNotFound) {
		return nil
	}
	return oops.In("release").With("project_id", projectID, "tag", tagName).Wrapf(err, "check existing release")
}

func (s *Service) ensureReleaseTag(ctx context.Context, project projectdomain.Project, projectID int64, input CreateReleaseInput) (gitports.Tag, error) {
	tag, ok, err := s.findTag(ctx, project, input.TagName)
	if err != nil {
		return gitports.Tag{}, err
	}
	if ok {
		return tag, nil
	}
	if !input.CreateTag {
		return gitports.Tag{}, apperror.NotFound("release tag not found", gitports.ErrReferenceNotFound)
	}
	return s.CreateTag(ctx, projectID, CreateTagInput{Name: input.TagName, SourceRef: input.SourceRef})
}

func (s *Service) findTag(ctx context.Context, project projectdomain.Project, tagName string) (gitports.Tag, bool, error) {
	tags, err := s.gitRepository.ListTags(ctx, repositoryPath(project))
	if err != nil {
		return gitports.Tag{}, false, mapGitError(err)
	}
	for _, tag := range tags {
		if tag.Name == strings.TrimSpace(tagName) {
			return tag, true, nil
		}
	}
	return gitports.Tag{}, false, nil
}

func (s *Service) tagsByName(ctx context.Context, project projectdomain.Project) (map[string]gitports.Tag, error) {
	tags, err := s.gitRepository.ListTags(ctx, repositoryPath(project))
	if err != nil {
		return map[string]gitports.Tag{}, mapGitError(err)
	}
	items := make(map[string]gitports.Tag, len(tags))
	for _, tag := range tags {
		items[tag.Name] = tag
	}
	return items, nil
}

func tagPointer(items map[string]gitports.Tag, name string) *gitports.Tag {
	item, ok := items[name]
	if !ok {
		return nil
	}
	return &item
}

func normalizeCreateReleaseInput(projectID int64, input CreateReleaseInput) (CreateReleaseInput, error) {
	tagName := strings.TrimSpace(input.TagName)
	name := strings.TrimSpace(input.Name)
	if tagName == "" {
		return CreateReleaseInput{}, apperror.BadRequest("tag name is required", oops.In("release").With("project_id", projectID).New("tag name is required"))
	}
	if name == "" {
		name = tagName
	}
	releasedAt := time.Now().UTC()
	if input.ReleasedAt != nil && !input.ReleasedAt.IsZero() {
		releasedAt = input.ReleasedAt.UTC()
	}
	return CreateReleaseInput{
		TagName:         tagName,
		Name:            name,
		Description:     strings.TrimSpace(input.Description),
		SourceRef:       strings.TrimSpace(input.SourceRef),
		CreateTag:       input.CreateTag,
		ReleasedAt:      &releasedAt,
		CreatedByUserID: input.CreatedByUserID,
	}, nil
}

func normalizeReleaseLinkInput(projectID, releaseID int64, input CreateReleaseLinkInput) (CreateReleaseLinkInput, error) {
	name := strings.TrimSpace(input.Name)
	rawURL := strings.TrimSpace(input.URL)
	if name == "" || rawURL == "" {
		return CreateReleaseLinkInput{}, apperror.BadRequest("release link name and url are required", oops.In("release").With("project_id", projectID, "release_id", releaseID).New("release link name and url are required"))
	}
	parsed, err := url.Parse(rawURL)
	if err != nil {
		return CreateReleaseLinkInput{}, apperror.BadRequest("release link url is invalid", oops.In("release").With("project_id", projectID, "release_id", releaseID, "url", rawURL).Wrapf(err, "parse release link url"))
	}
	if parsed.Scheme == "" || parsed.Host == "" {
		return CreateReleaseLinkInput{}, apperror.BadRequest("release link url is invalid", oops.In("release").With("project_id", projectID, "release_id", releaseID, "url", rawURL).New("release link url is invalid"))
	}
	linkType := strings.TrimSpace(input.LinkType)
	if linkType == "" {
		linkType = "other"
	}
	return CreateReleaseLinkInput{Name: name, URL: rawURL, LinkType: linkType}, nil
}

func repositoryPath(project projectdomain.Project) string {
	return project.FullPath + ".git"
}
