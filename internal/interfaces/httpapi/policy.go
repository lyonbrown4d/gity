package httpapi

import (
	"context"
	"net/http"
	"strings"

	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/arcgolabs/httpx"
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
	return projectPolicy[I, O]("require_project_read", authRuntime, resolver, true)
}

func RequireProjectWrite[I ProjectInput, O any](authRuntime *infraauth.Runtime, resolver ProjectScopeResolver) httpx.RoutePolicy[I, O] {
	return projectPolicy[I, O]("require_project_write", authRuntime, resolver, false)
}

func projectPolicy[I ProjectInput, O any](name string, authRuntime *infraauth.Runtime, resolver ProjectScopeResolver, read bool) httpx.RoutePolicy[I, O] {
	return httpx.RoutePolicy[I, O]{
		Name:      name,
		Operation: markBearerProtected,
		Wrap: func(next httpx.TypedHandler[I, O]) httpx.TypedHandler[I, O] {
			if next == nil {
				return nil
			}
			return func(ctx context.Context, input *I) (*O, error) {
				if resolver == nil {
					return nil, httpx.NewError(http.StatusInternalServerError, "project scope resolver is not configured")
				}
				scope, err := resolver(ctx, projectID(input))
				if err != nil {
					return nil, err
				}
				authorization := authorizationHeader(input)
				if read && strings.EqualFold(strings.TrimSpace(scope.Visibility), "public") && strings.TrimSpace(authorization) == "" {
					return next(ctx, input)
				}
				principal, ok, err := authenticateHeader(ctx, authRuntime, authorization)
				if err != nil {
					return nil, err
				}
				if !ok {
					return nil, httpx.NewError(http.StatusUnauthorized, "authentication required")
				}
				allowed := false
				if read {
					allowed, err = authRuntime.CanReadProject(ctx, principal, scope)
				} else {
					allowed, err = authRuntime.CanWriteProject(ctx, principal, scope)
				}
				if err != nil {
					return nil, httpx.NewError(http.StatusForbidden, "authorization failed", err)
				}
				if !allowed {
					return nil, httpx.NewError(http.StatusForbidden, "forbidden")
				}
				return next(ctx, input)
			}
		},
	}
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
