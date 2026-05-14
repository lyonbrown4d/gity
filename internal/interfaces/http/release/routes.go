// Package release exposes release and tag HTTP APIs.
package release

import (
	"context"
	"strconv"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	releaseservice "github.com/lyonbrown4d/gity/internal/application/release"
	releasedomain "github.com/lyonbrown4d/gity/internal/domain/release"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type Endpoint struct {
	service        *releaseservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
}

func NewEndpoint(service *releaseservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Releases", "Releases", "Project release and repository tag APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)
	readRepository := httpapi.RequireProjectActionRoute[projectReleaseInput, releaseOutput]("require_release_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)
	readTag := httpapi.RequireProjectActionRoute[projectTagInput, releaseOutput]("require_repository_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)
	adminTag := httpapi.RequireProjectActionRoute[createTagInput, releaseOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	createRelease := httpapi.RequireProjectActionRoute[createReleaseInput, releaseOutput]("require_release_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	updateRelease := httpapi.RequireProjectActionRoute[releaseByIDInput, releaseOutput]("require_release_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	linkRelease := httpapi.RequireProjectActionRoute[createReleaseLinkInput, releaseOutput]("require_release_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	deleteLink := httpapi.RequireProjectActionRoute[releaseLinkInput, releaseOutput]("require_release_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/repository/tags", e.listTags, readTag),
		httpapi.Post("/projects/{id}/repository/tags", e.createTag, adminTag),
		httpapi.Delete("/projects/{id}/repository/tags", e.deleteTag, httpapi.RequireProjectActionRoute[deleteTagInput, releaseOutput]("require_repository_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)),
		httpapi.Get("/projects/{id}/releases", e.listReleases, readRepository),
		httpapi.Post("/projects/{id}/releases", e.createRelease, createRelease),
		httpapi.Get("/projects/{id}/releases/{release_id}", e.getRelease, httpapi.RequireProjectActionRoute[releaseByIDInput, releaseOutput]("require_release_read", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryRead)),
		httpapi.Patch("/projects/{id}/releases/{release_id}", e.updateRelease, updateRelease),
		httpapi.Delete("/projects/{id}/releases/{release_id}", e.deleteRelease, updateRelease),
		httpapi.Post("/projects/{id}/releases/{release_id}/links", e.createReleaseLink, linkRelease),
		httpapi.Delete("/projects/{id}/releases/{release_id}/links/{link_id}", e.deleteReleaseLink, deleteLink),
		httpapi.Get("/repos/{id}/releases", e.listReleases, readRepository, httpapi.DeprecatedRoute[projectReleaseInput, releaseOutput]("Use GET /projects/{id}/releases instead.")),
		httpapi.Post("/repos/{id}/releases", e.createRelease, createRelease, httpapi.DeprecatedRoute[createReleaseInput, releaseOutput]("Use POST /projects/{id}/releases instead.")),
		httpapi.Get("/repos/{id}/repository/tags", e.listTags, readTag, httpapi.DeprecatedRoute[projectTagInput, releaseOutput]("Use GET /projects/{id}/repository/tags instead.")),
		httpapi.Post("/repos/{id}/repository/tags", e.createTag, adminTag, httpapi.DeprecatedRoute[createTagInput, releaseOutput]("Use POST /projects/{id}/repository/tags instead.")),
	)
}

func (e *Endpoint) listTags(ctx context.Context, in *projectTagInput) (*releaseOutput, error) {
	items, err := e.service.ListTags(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: items}, nil
}

func (e *Endpoint) createTag(ctx context.Context, in *createTagInput) (*releaseOutput, error) {
	item, err := e.service.CreateTag(ctx, in.ProjectID, releaseservice.CreateTagInput{Name: in.Body.Name, SourceRef: in.Body.SourceRef})
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: item}, nil
}

func (e *Endpoint) deleteTag(ctx context.Context, in *deleteTagInput) (*releaseOutput, error) {
	if err := e.service.DeleteTag(ctx, in.ProjectID, in.Name); err != nil {
		return nil, err
	}
	return &releaseOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func (e *Endpoint) listReleases(ctx context.Context, in *projectReleaseInput) (*releaseOutput, error) {
	items, err := e.service.ListReleases(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item releaseservice.ReleaseDetail) releaseDetailView {
		return toReleaseDetailView(item)
	}).Values()}, nil
}

func (e *Endpoint) getRelease(ctx context.Context, in *releaseByIDInput) (*releaseOutput, error) {
	item, err := e.service.GetRelease(ctx, in.ProjectID, in.ReleaseID)
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: toReleaseDetailView(item)}, nil
}

func (e *Endpoint) createRelease(ctx context.Context, in *createReleaseInput) (*releaseOutput, error) {
	actorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, in.Body.CreatedByUserID)
	if err != nil {
		return nil, err
	}
	releasedAt, err := parseOptionalTime(in.Body.ReleasedAt)
	if err != nil {
		return nil, err
	}
	item, err := e.service.CreateRelease(ctx, in.ProjectID, releaseservice.CreateReleaseInput{
		TagName:         in.Body.TagName,
		Name:            in.Body.Name,
		Description:     in.Body.Description,
		SourceRef:       in.Body.SourceRef,
		CreateTag:       in.Body.CreateTag,
		ReleasedAt:      releasedAt,
		CreatedByUserID: actorUserID,
	})
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: toReleaseDetailView(item)}, nil
}

func (e *Endpoint) updateRelease(ctx context.Context, in *releaseByIDInput) (*releaseOutput, error) {
	releasedAt, err := parseOptionalTime(in.Body.ReleasedAt)
	if err != nil {
		return nil, err
	}
	item, err := e.service.UpdateRelease(ctx, in.ProjectID, in.ReleaseID, releaseservice.UpdateReleaseInput{
		Name:        optionalString(in.Body.Name),
		Description: optionalString(in.Body.Description),
		ReleasedAt:  releasedAt,
	})
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: toReleaseDetailView(item)}, nil
}

func (e *Endpoint) deleteRelease(ctx context.Context, in *releaseByIDInput) (*releaseOutput, error) {
	if err := e.service.DeleteRelease(ctx, in.ProjectID, in.ReleaseID, in.DeleteTag); err != nil {
		return nil, err
	}
	return &releaseOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func (e *Endpoint) createReleaseLink(ctx context.Context, in *createReleaseLinkInput) (*releaseOutput, error) {
	item, err := e.service.CreateReleaseLink(ctx, in.ProjectID, in.ReleaseID, releaseservice.CreateReleaseLinkInput{Name: in.Body.Name, URL: in.Body.URL, LinkType: in.Body.LinkType})
	if err != nil {
		return nil, err
	}
	return &releaseOutput{Body: toReleaseLinkView(item)}, nil
}

func (e *Endpoint) deleteReleaseLink(ctx context.Context, in *releaseLinkInput) (*releaseOutput, error) {
	if err := e.service.DeleteReleaseLink(ctx, in.ProjectID, in.ReleaseID, in.LinkID); err != nil {
		return nil, err
	}
	return &releaseOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func parseOptionalTime(value string) (*time.Time, error) {
	if value == "" {
		return nil, nil
	}
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return nil, err
	}
	return &parsed, nil
}

func optionalString(value *string) *string {
	if value == nil {
		return nil
	}
	normalized := *value
	return &normalized
}

func toReleaseDetailView(item releaseservice.ReleaseDetail) releaseDetailView {
	return releaseDetailView{
		Release: toReleaseView(item.Release),
		Links: collectionlist.MapList(collectionlist.NewList(item.Links...), func(_ int, link releasedomain.ProjectReleaseLink) releaseLinkView {
			return toReleaseLinkView(link)
		}).Values(),
		Tag: item.Tag,
	}
}

func toReleaseView(item releasedomain.ProjectRelease) releaseView {
	return releaseView{
		ID:              strconv.FormatInt(item.ID, 10),
		ProjectID:       strconv.FormatInt(item.ProjectID, 10),
		TagName:         item.TagName,
		Name:            item.Name,
		Description:     item.Description,
		CreatedByUserID: strconv.FormatInt(item.CreatedByUserID, 10),
		ReleasedAt:      formatTime(item.ReleasedAt),
		CreatedAt:       formatTime(item.CreatedAt),
		UpdatedAt:       formatTime(item.UpdatedAt),
	}
}

func toReleaseLinkView(item releasedomain.ProjectReleaseLink) releaseLinkView {
	return releaseLinkView{
		ID:               strconv.FormatInt(item.ID, 10),
		ProjectReleaseID: strconv.FormatInt(item.ProjectReleaseID, 10),
		Name:             item.Name,
		URL:              item.URL,
		LinkType:         item.LinkType,
		CreatedAt:        formatTime(item.CreatedAt),
		UpdatedAt:        formatTime(item.UpdatedAt),
	}
}

func formatTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}
