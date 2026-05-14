package user_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	user "github.com/lyonbrown4d/gity/internal/interfaces/http/user"
)

func TestEndpointRegistersUserRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(user.NewEndpoint(nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/users")
	assertRoute(t, server, http.MethodGet, "/api/v1/users/me")
	assertRoute(t, server, http.MethodPatch, "/api/v1/users/me")
	assertRoute(t, server, http.MethodPost, "/api/v1/users/{id}/tokens")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
