package projectcredential

import (
	"context"
	"strconv"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	projectcredentialservice "github.com/lyonbrown4d/gity/internal/application/project_credential"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type Endpoint struct {
	service        *projectcredentialservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
}

func NewEndpoint(service *projectcredentialservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Project Credentials", "Project Credentials", "Project access token, deploy token and deploy key APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)
	admin := httpapi.RequireProjectActionRoute[projectCredentialInput, credentialOutput]("require_project_credential_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	createToken := httpapi.RequireProjectActionRoute[createProjectTokenInput, credentialOutput]("require_project_credential_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	tokenAdmin := httpapi.RequireProjectActionRoute[projectTokenInput, credentialOutput]("require_project_credential_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	createKey := httpapi.RequireProjectActionRoute[createDeployKeyInput, credentialOutput]("require_project_credential_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)
	keyAdmin := httpapi.RequireProjectActionRoute[deployKeyInput, credentialOutput]("require_project_credential_admin", e.authRuntime, projectScope, infraauth.ProjectActionRepositoryAdmin)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/access-tokens", e.listProjectAccessTokens, admin),
		httpapi.Post("/projects/{id}/access-tokens", e.createProjectAccessToken, createToken),
		httpapi.Delete("/projects/{id}/access-tokens/{token_id}", e.revokeProjectAccessToken, tokenAdmin),
		httpapi.Get("/projects/{id}/deploy-tokens", e.listDeployTokens, admin),
		httpapi.Post("/projects/{id}/deploy-tokens", e.createDeployToken, createToken),
		httpapi.Delete("/projects/{id}/deploy-tokens/{token_id}", e.revokeDeployToken, tokenAdmin),
		httpapi.Get("/projects/{id}/deploy-keys", e.listDeployKeys, admin),
		httpapi.Post("/projects/{id}/deploy-keys", e.createDeployKey, createKey),
		httpapi.Delete("/projects/{id}/deploy-keys/{key_id}", e.deleteDeployKey, keyAdmin),
	)
}

func (e *Endpoint) listProjectAccessTokens(ctx context.Context, in *projectCredentialInput) (*credentialOutput, error) {
	items, err := e.service.ListProjectAccessTokens(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: tokenViews(items)}, nil
}

func (e *Endpoint) createProjectAccessToken(ctx context.Context, in *createProjectTokenInput) (*credentialOutput, error) {
	item, err := e.createToken(ctx, in, e.service.CreateProjectAccessToken)
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: createdProjectTokenView{ProjectToken: toProjectTokenView(item), Token: item.Token}}, nil
}

func (e *Endpoint) revokeProjectAccessToken(ctx context.Context, in *projectTokenInput) (*credentialOutput, error) {
	if err := e.service.RevokeProjectAccessToken(ctx, in.ProjectID, in.TokenID); err != nil {
		return nil, err
	}
	return &credentialOutput{Body: map[string]any{"status": "revoked"}}, nil
}

func (e *Endpoint) listDeployTokens(ctx context.Context, in *projectCredentialInput) (*credentialOutput, error) {
	items, err := e.service.ListDeployTokens(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: tokenViews(items)}, nil
}

func (e *Endpoint) createDeployToken(ctx context.Context, in *createProjectTokenInput) (*credentialOutput, error) {
	item, err := e.createToken(ctx, in, e.service.CreateDeployToken)
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: createdProjectTokenView{ProjectToken: toProjectTokenView(item), Token: item.Token}}, nil
}

func (e *Endpoint) revokeDeployToken(ctx context.Context, in *projectTokenInput) (*credentialOutput, error) {
	if err := e.service.RevokeDeployToken(ctx, in.ProjectID, in.TokenID); err != nil {
		return nil, err
	}
	return &credentialOutput{Body: map[string]any{"status": "revoked"}}, nil
}

func (e *Endpoint) listDeployKeys(ctx context.Context, in *projectCredentialInput) (*credentialOutput, error) {
	items, err := e.service.ListDeployKeys(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item identity.ProjectDeployKey) deployKeyView {
		return toDeployKeyView(item)
	}).Values()}, nil
}

func (e *Endpoint) createDeployKey(ctx context.Context, in *createDeployKeyInput) (*credentialOutput, error) {
	actorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, in.Body.CreatedByUserID)
	if err != nil {
		return nil, err
	}
	item, err := e.service.CreateDeployKey(ctx, in.ProjectID, projectcredentialservice.CreateDeployKeyInput{
		Title:           in.Body.Title,
		PublicKey:       in.Body.PublicKey,
		CanPush:         in.Body.CanPush,
		CreatedByUserID: actorUserID,
	})
	if err != nil {
		return nil, err
	}
	return &credentialOutput{Body: toDeployKeyView(item)}, nil
}

func (e *Endpoint) deleteDeployKey(ctx context.Context, in *deployKeyInput) (*credentialOutput, error) {
	if err := e.service.DeleteDeployKey(ctx, in.ProjectID, in.KeyID); err != nil {
		return nil, err
	}
	return &credentialOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func (e *Endpoint) createToken(ctx context.Context, in *createProjectTokenInput, create func(context.Context, int64, projectcredentialservice.CreateTokenInput) (identity.ProjectAccessToken, error)) (identity.ProjectAccessToken, error) {
	actorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, in.Body.CreatedByUserID)
	if err != nil {
		return identity.ProjectAccessToken{}, err
	}
	expiresAt, err := parseOptionalTime(in.Body.ExpiresAt)
	if err != nil {
		return identity.ProjectAccessToken{}, err
	}
	return create(ctx, in.ProjectID, projectcredentialservice.CreateTokenInput{
		Name:            in.Body.Name,
		Username:        in.Body.Username,
		Scopes:          in.Body.Scopes,
		CreatedByUserID: actorUserID,
		ExpiresAt:       expiresAt,
	})
}

func tokenViews(items []identity.ProjectAccessToken) []projectTokenView {
	return collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item identity.ProjectAccessToken) projectTokenView {
		return toProjectTokenView(item)
	}).Values()
}

func toProjectTokenView(item identity.ProjectAccessToken) projectTokenView {
	return projectTokenView{
		ID:              strconv.FormatInt(item.ID, 10),
		ProjectID:       strconv.FormatInt(item.ProjectID, 10),
		Kind:            item.Kind,
		Name:            item.Name,
		Username:        item.Username,
		Scopes:          splitScopes(item.Scopes),
		CreatedByUserID: strconv.FormatInt(item.CreatedByUserID, 10),
		ExpiresAt:       formatTime(item.ExpiresAt),
		RevokedAt:       formatTime(item.RevokedAt),
		LastUsedAt:      formatTime(item.LastUsedAt),
		Active:          item.Active(time.Now().UTC()),
		CreatedAt:       formatTime(item.CreatedAt),
		UpdatedAt:       formatTime(item.UpdatedAt),
	}
}

func toDeployKeyView(item identity.ProjectDeployKey) deployKeyView {
	return deployKeyView{
		ID:              strconv.FormatInt(item.ID, 10),
		ProjectID:       strconv.FormatInt(item.ProjectID, 10),
		Title:           item.Title,
		Fingerprint:     item.Fingerprint,
		PublicKey:       item.PublicKey,
		CanPush:         item.PushEnabled(),
		CreatedByUserID: strconv.FormatInt(item.CreatedByUserID, 10),
		LastUsedAt:      formatTime(item.LastUsedAt),
		CreatedAt:       formatTime(item.CreatedAt),
		UpdatedAt:       formatTime(item.UpdatedAt),
	}
}

func splitScopes(value string) []string {
	parts := strings.Split(value, ",")
	scopes := make([]string, 0, len(parts))
	for _, part := range parts {
		scope := strings.TrimSpace(part)
		if scope != "" {
			scopes = append(scopes, scope)
		}
	}
	return scopes
}

func parseOptionalTime(value string) (time.Time, error) {
	if strings.TrimSpace(value) == "" {
		return time.Time{}, nil
	}
	return time.Parse(time.RFC3339, value)
}

func formatTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format(time.RFC3339)
}
