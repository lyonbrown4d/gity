// Package release implements project release and tag management.
package release

import (
	"context"
	"errors"
	"log/slog"
	"strings"

	collectionx "github.com/arcgolabs/collectionx/list"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
	"github.com/samber/oops"
)

func NewService(logger *slog.Logger, projectRepo gitports.ProjectRepository, releaseRepo gitports.ProjectReleaseRepository, linkRepo gitports.ProjectReleaseLinkRepository, gitRepository gitports.GitRepository, gitRunner gitports.GitRunner) *Service {
	return &Service{logger: logger, projectRepo: projectRepo, releaseRepo: releaseRepo, linkRepo: linkRepo, gitRepository: gitRepository, gitRunner: gitRunner}
}

func (s *Service) ListTags(ctx context.Context, projectID int64) ([]gitports.Tag, error) {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return nil, err
	}
	tags, err := s.gitRepository.ListTags(ctx, repositoryPath(project))
	if err != nil {
		return nil, mapGitError(err)
	}
	return tags, nil
}

func (s *Service) CreateTag(ctx context.Context, projectID int64, input CreateTagInput) (gitports.Tag, error) {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return gitports.Tag{}, err
	}
	tagName := strings.TrimSpace(input.Name)
	if tagName == "" {
		return gitports.Tag{}, apperror.BadRequest("tag name is required", oops.In("release").With("project_id", projectID).New("tag name is required"))
	}
	sourceRef := strings.TrimSpace(input.SourceRef)
	if sourceRef == "" {
		sourceRef = project.DefaultBranch
	}
	if err := s.gitRunner.CreateTag(ctx, repositoryPath(project), tagName, sourceRef); err != nil {
		return gitports.Tag{}, mapGitExecError(err)
	}
	return s.getTag(ctx, project, tagName)
}

func (s *Service) DeleteTag(ctx context.Context, projectID int64, tagName string) error {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return err
	}
	tagName = strings.TrimSpace(tagName)
	if tagName == "" {
		return apperror.BadRequest("tag name is required", oops.In("release").With("project_id", projectID).New("tag name is required"))
	}
	if err := s.gitRunner.DeleteTag(ctx, repositoryPath(project), tagName); err != nil {
		return mapGitExecError(err)
	}
	return nil
}

func (s *Service) ListReleases(ctx context.Context, projectID int64) ([]ReleaseDetail, error) {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return nil, err
	}
	releases, err := s.releaseRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("release").With("project_id", projectID).Wrapf(err, "list releases")
	}
	releaseValues := releases.Values()
	ids := collectionx.MapList(releases, func(_ int, item releasedomain.ProjectRelease) int64 {
		return item.ID
	}).Values()
	links, err := s.linkRepo.ListByReleaseIDs(ctx, ids...)
	if err != nil {
		return nil, oops.In("release").With("project_id", projectID).Wrapf(err, "list release links")
	}
	linksByRelease := mappingx.GroupByList(links, func(_ int, item releasedomain.ProjectReleaseLink) int64 {
		return item.ProjectReleaseID
	})
	tagsByName, err := s.tagsByName(ctx, project)
	if err != nil {
		return nil, err
	}
	details := make([]ReleaseDetail, 0, len(releaseValues))
	for index := range releaseValues {
		item := releaseValues[index]
		details = append(details, ReleaseDetail{Release: item, Links: linksByRelease.GetCopy(item.ID), Tag: tagPointer(tagsByName, item.TagName)})
	}
	return details, nil
}

func (s *Service) GetRelease(ctx context.Context, projectID, releaseID int64) (ReleaseDetail, error) {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return ReleaseDetail{}, err
	}
	item, err := s.loadRelease(ctx, projectID, releaseID)
	if err != nil {
		return ReleaseDetail{}, err
	}
	links, err := s.linkRepo.ListByReleaseID(ctx, item.ID)
	if err != nil {
		return ReleaseDetail{}, oops.In("release").With("project_id", projectID, "release_id", releaseID).Wrapf(err, "list release links")
	}
	tag, ok, err := s.findTag(ctx, project, item.TagName)
	if err != nil {
		return ReleaseDetail{}, err
	}
	if !ok {
		return ReleaseDetail{Release: item, Links: links.Values()}, nil
	}
	return ReleaseDetail{Release: item, Links: links.Values(), Tag: &tag}, nil
}

func (s *Service) CreateRelease(ctx context.Context, projectID int64, input CreateReleaseInput) (ReleaseDetail, error) {
	project, err := s.loadProject(ctx, projectID)
	if err != nil {
		return ReleaseDetail{}, err
	}
	normalized, err := normalizeCreateReleaseInput(projectID, input)
	if err != nil {
		return ReleaseDetail{}, err
	}
	if existsErr := s.ensureReleaseDoesNotExist(ctx, projectID, normalized.TagName); existsErr != nil {
		return ReleaseDetail{}, existsErr
	}
	tag, err := s.ensureReleaseTag(ctx, project, projectID, normalized)
	if err != nil {
		return ReleaseDetail{}, err
	}
	item, err := s.releaseRepo.Create(ctx, gitports.CreateProjectReleaseInput{
		ProjectID:       projectID,
		TagName:         normalized.TagName,
		Name:            normalized.Name,
		Description:     normalized.Description,
		CreatedByUserID: normalized.CreatedByUserID,
		ReleasedAt:      *normalized.ReleasedAt,
	})
	if err != nil {
		return ReleaseDetail{}, oops.In("release").With("project_id", projectID, "tag", normalized.TagName).Wrapf(err, "create release")
	}
	return ReleaseDetail{Release: item, Links: []releasedomain.ProjectReleaseLink{}, Tag: &tag}, nil
}

func (s *Service) UpdateRelease(ctx context.Context, projectID, releaseID int64, input UpdateReleaseInput) (ReleaseDetail, error) {
	if _, err := s.loadRelease(ctx, projectID, releaseID); err != nil {
		return ReleaseDetail{}, err
	}
	if err := s.releaseRepo.UpdateByID(ctx, releaseID, gitports.UpdateProjectReleaseInput{Name: input.Name, Description: input.Description, ReleasedAt: input.ReleasedAt}); err != nil {
		return ReleaseDetail{}, oops.In("release").With("project_id", projectID, "release_id", releaseID).Wrapf(err, "update release")
	}
	return s.GetRelease(ctx, projectID, releaseID)
}

func (s *Service) DeleteRelease(ctx context.Context, projectID, releaseID int64, deleteTag bool) error {
	item, err := s.loadRelease(ctx, projectID, releaseID)
	if err != nil {
		return err
	}
	if err := s.releaseRepo.DeleteByID(ctx, releaseID); err != nil {
		return oops.In("release").With("project_id", projectID, "release_id", releaseID).Wrapf(err, "delete release")
	}
	if deleteTag {
		if err := s.DeleteTag(ctx, projectID, item.TagName); err != nil && !errors.Is(err, gitports.ErrReferenceNotFound) {
			return err
		}
	}
	return nil
}

func (s *Service) CreateReleaseLink(ctx context.Context, projectID, releaseID int64, input CreateReleaseLinkInput) (releasedomain.ProjectReleaseLink, error) {
	if _, err := s.loadRelease(ctx, projectID, releaseID); err != nil {
		return releasedomain.ProjectReleaseLink{}, err
	}
	normalized, err := normalizeReleaseLinkInput(projectID, releaseID, input)
	if err != nil {
		return releasedomain.ProjectReleaseLink{}, err
	}
	item, err := s.linkRepo.Create(ctx, gitports.CreateProjectReleaseLinkInput{
		ProjectReleaseID: releaseID,
		Name:             normalized.Name,
		URL:              normalized.URL,
		LinkType:         normalized.LinkType,
	})
	if err != nil {
		return releasedomain.ProjectReleaseLink{}, oops.In("release").With("project_id", projectID, "release_id", releaseID).Wrapf(err, "create release link")
	}
	return item, nil
}

func (s *Service) DeleteReleaseLink(ctx context.Context, projectID, releaseID, linkID int64) error {
	if _, err := s.loadRelease(ctx, projectID, releaseID); err != nil {
		return err
	}
	if err := s.linkRepo.DeleteByID(ctx, linkID); err != nil {
		return oops.In("release").With("project_id", projectID, "release_id", releaseID, "link_id", linkID).Wrapf(err, "delete release link")
	}
	return nil
}
