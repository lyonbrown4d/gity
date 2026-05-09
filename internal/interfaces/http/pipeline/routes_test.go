package pipeline_test

import (
	"net/http"
	"testing"

	pipeline "github.com/DaiYuANg/gity/internal/interfaces/http/pipeline"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalPipelineRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(pipeline.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/pipelines")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/pipelines")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/ci/lint")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/pipelines/{pipeline_id}/retry")
}

func TestEndpointRegistersDeprecatedRepoPipelineAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(pipeline.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/pipelines")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/pipelines")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/ci/lint")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/pipelines/{pipeline_id}/retry")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
