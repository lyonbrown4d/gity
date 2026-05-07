package project

import (
	"net/http"
	"testing"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalProjectRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(NewEndpoint(nil, config.Settings{}, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/repository/branches")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/repository/files")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/repository/branches/{branch_name}/protect")
}

func TestEndpointRegistersDeprecatedRepoAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(NewEndpoint(nil, config.Settings{}, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/branches")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/file-commits")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/branches/{branch_name}/protect")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method string, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
