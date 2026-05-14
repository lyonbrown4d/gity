package project_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	"github.com/lyonbrown4d/gity/internal/config"
	project "github.com/lyonbrown4d/gity/internal/interfaces/http/project"
)

func TestEndpointRegistersCanonicalProjectRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(project.NewEndpoint(nil, config.Settings{}, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/repository/branches")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/repository/branches/{branch_name}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/repository/branch-protections")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/repository/branch-protections/{branch_name}")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/repository/files")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/repository/branches/{branch_name}/protect")
}

func TestEndpointRegistersDeprecatedRepoAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(project.NewEndpoint(nil, config.Settings{}, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/branches")
	assertRoute(t, server, http.MethodDelete, "/api/v1/repos/{id}/branches/{branch_name}")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/file-commits")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/branches/{branch_name}/protect")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
