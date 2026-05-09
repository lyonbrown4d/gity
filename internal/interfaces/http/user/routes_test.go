package user_test

import (
	"net/http"
	"testing"

	user "github.com/DaiYuANg/gity/internal/interfaces/http/user"
	"github.com/arcgolabs/httpx"
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
