package httpapi

import (
	"context"
	"net/http"
	"strings"

	"github.com/arcgolabs/httpx"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
)

type AuthorizationInput interface {
	AuthorizationHeader() string
}

type ProjectInput interface {
	AuthorizationInput
	ProjectIDValue() int64
}

type ProjectScopeResolver func(context.Context, int64) (infraauth.ProjectScope, error)

func ActorUserID(ctx context.Context, authRuntime *infraauth.Runtime, authorization string, fallback int64) (int64, error) {
	if strings.TrimSpace(authorization) != "" {
		principal, ok, err := authenticateHeader(ctx, authRuntime, authorization)
		if err != nil {
			return 0, err
		}
		if ok {
			return principal.UserID, nil
		}
	}
	if fallback > 0 {
		return fallback, nil
	}
	principal, ok, err := authenticateHeader(ctx, authRuntime, authorization)
	if err != nil {
		return 0, err
	}
	if !ok {
		return 0, httpx.NewError(http.StatusUnauthorized, "authentication required")
	}
	return principal.UserID, nil
}

func RequireUser[I AuthorizationInput, O any](authRuntime *infraauth.Runtime) httpx.RoutePolicy[I, O] {
	return httpx.RoutePolicy[I, O]{
		Name:      "require_user",
		Operation: markBearerProtected,
		Wrap: func(next httpx.TypedHandler[I, O]) httpx.TypedHandler[I, O] {
			if next == nil {
				return nil
			}
			return func(ctx context.Context, input *I) (*O, error) {
				if _, ok, err := authenticateHeader(ctx, authRuntime, authorizationHeader(input)); err != nil {
					return nil, err
				} else if !ok {
					return nil, httpx.NewError(http.StatusUnauthorized, "authentication required")
				}
				return next(ctx, input)
			}
		},
	}
}

func RequireProjectRead[I ProjectInput, O any](authRuntime *infraauth.Runtime, resolver ProjectScopeResolver) httpx.RoutePolicy[I, O] {
	return projectPolicy[I, O]("require_project_read", authRuntime, resolver, infraauth.ProjectActionRead, true)
}

func RequireProjectWrite[I ProjectInput, O any](authRuntime *infraauth.Runtime, resolver ProjectScopeResolver) httpx.RoutePolicy[I, O] {
	return RequireProjectAction[I, O]("require_project_write", authRuntime, resolver, infraauth.ProjectActionWrite)
}

func RequireProjectAction[I ProjectInput, O any](name string, authRuntime *infraauth.Runtime, resolver ProjectScopeResolver, action string) httpx.RoutePolicy[I, O] {
	return projectPolicy[I, O](name, authRuntime, resolver, action, isAnonymousProjectReadAction(action))
}

func isAnonymousProjectReadAction(action string) bool {
	switch action {
	case infraauth.ProjectActionRead,
		infraauth.ProjectActionRepositoryRead,
		infraauth.ProjectActionPackageRead,
		infraauth.ProjectActionWikiRead:
		return true
	default:
		return false
	}
}

func projectPolicy[I ProjectInput, O any](name string, authRuntime *infraauth.Runtime, resolver ProjectScopeResolver, action string, allowAnonymousPublicRead bool) httpx.RoutePolicy[I, O] {
	return httpx.RoutePolicy[I, O]{
		Name:      name,
		Operation: markBearerProtected,
		Wrap: func(next httpx.TypedHandler[I, O]) httpx.TypedHandler[I, O] {
			if next == nil {
				return nil
			}
			return func(ctx context.Context, input *I) (*O, error) {
				return authorizeProjectRequest(ctx, input, next, authRuntime, resolver, action, allowAnonymousPublicRead)
			}
		},
	}
}

func authorizeProjectRequest[I ProjectInput, O any](ctx context.Context, input *I, next httpx.TypedHandler[I, O], authRuntime *infraauth.Runtime, resolver ProjectScopeResolver, action string, allowAnonymousPublicRead bool) (*O, error) {
	scope, err := resolveProjectScope(ctx, input, resolver)
	if err != nil {
		return nil, err
	}
	authorization := authorizationHeader(input)
	if canReadPublicProjectAnonymously(allowAnonymousPublicRead, scope, authorization) {
		return next(ctx, input)
	}
	principal, err := requirePrincipal(ctx, authRuntime, authorization)
	if err != nil {
		return nil, err
	}
	if err := authorizeProjectAccess(ctx, authRuntime, principal, scope, action); err != nil {
		return nil, err
	}
	return next(ctx, input)
}

func resolveProjectScope[I ProjectInput](ctx context.Context, input *I, resolver ProjectScopeResolver) (infraauth.ProjectScope, error) {
	if resolver == nil {
		return infraauth.ProjectScope{}, httpx.NewError(http.StatusInternalServerError, "project scope resolver is not configured")
	}
	return resolver(ctx, projectID(input))
}

func canReadPublicProjectAnonymously(read bool, scope infraauth.ProjectScope, authorization string) bool {
	return read && strings.EqualFold(strings.TrimSpace(scope.Visibility), "public") && strings.TrimSpace(authorization) == ""
}

func requirePrincipal(ctx context.Context, authRuntime *infraauth.Runtime, authorization string) (infraauth.Principal, error) {
	principal, ok, err := authenticateHeader(ctx, authRuntime, authorization)
	if err != nil {
		return infraauth.Principal{}, err
	}
	if !ok {
		return infraauth.Principal{}, httpx.NewError(http.StatusUnauthorized, "authentication required")
	}
	return principal, nil
}

func authorizeProjectAccess(ctx context.Context, authRuntime *infraauth.Runtime, principal infraauth.Principal, scope infraauth.ProjectScope, action string) error {
	allowed, err := authRuntime.CanProjectAction(ctx, principal, scope, action)
	if err != nil {
		return httpx.NewError(http.StatusForbidden, "authorization failed", err)
	}
	if !allowed {
		return httpx.NewError(http.StatusForbidden, "forbidden")
	}
	return nil
}

func authenticateHeader(ctx context.Context, authRuntime *infraauth.Runtime, authorization string) (infraauth.Principal, bool, error) {
	if authRuntime == nil {
		return infraauth.Principal{}, false, httpx.NewError(http.StatusInternalServerError, "auth runtime is not configured")
	}
	principal, ok, err := authRuntime.AuthenticateHeader(ctx, authorization)
	if err != nil {
		return infraauth.Principal{}, false, httpx.NewError(http.StatusUnauthorized, "invalid credentials", err)
	}
	return principal, ok, nil
}

func authorizationHeader[I AuthorizationInput](input *I) string {
	if input == nil {
		return ""
	}
	return (*input).AuthorizationHeader()
}

func projectID[I ProjectInput](input *I) int64 {
	if input == nil {
		return 0
	}
	return (*input).ProjectIDValue()
}
