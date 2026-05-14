package runner_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	runner "github.com/lyonbrown4d/gity/internal/interfaces/http/runner"
)

func TestEndpointRegistersCanonicalRunnerRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(runner.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/runners")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/runners")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/runners/{runner_id}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/ci/variables")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/ci/variables")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/ci/variables/{key}")
	assertRoute(t, server, http.MethodPost, "/api/v1/runners/jobs/claim")
	assertRoute(t, server, http.MethodPost, "/api/v1/runners/jobs/{job_id}/trace")
	assertRoute(t, server, http.MethodPost, "/api/v1/runners/jobs/{job_id}/source-archive")
}

func TestEndpointRegistersDeprecatedRepoRunnerAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(runner.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/runners")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/runners")
	assertRoute(t, server, http.MethodDelete, "/api/v1/repos/{id}/runners/{runner_id}")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
