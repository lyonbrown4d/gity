package user

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersUserRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(NewEndpoint(nil, nil))

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
