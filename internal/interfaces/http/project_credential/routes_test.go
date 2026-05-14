package projectcredential_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	projectcredential "github.com/lyonbrown4d/gity/internal/interfaces/http/project_credential"
)

func TestEndpointRegistersProjectCredentialRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(projectcredential.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/access-tokens")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/access-tokens")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/access-tokens/{token_id}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/deploy-tokens")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/deploy-tokens")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/deploy-tokens/{token_id}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/deploy-keys")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/deploy-keys")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/deploy-keys/{key_id}")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
