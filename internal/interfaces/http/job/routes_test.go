package job_test

import (
	"net/http"
	"testing"

	job "github.com/DaiYuANg/gity/internal/interfaces/http/job"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalJobRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(job.NewEndpoint(nil, nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/jobs")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/jobs")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/jobs/{job_id}/cancel")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/jobs/{job_id}/artifacts/{artifact_id}")
}

func TestEndpointRegistersDeprecatedRepoJobAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(job.NewEndpoint(nil, nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/jobs")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/jobs")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/jobs/{job_id}/cancel")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/jobs/{job_id}/artifacts/{artifact_id}")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
