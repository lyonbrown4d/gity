package wiki_test

import (
	"net/http"
	"testing"

	wiki "github.com/DaiYuANg/gity/internal/interfaces/http/wiki"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalWikiRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(wiki.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/wiki/pages")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/wiki/pages")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/wiki/pages/{slug}")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/wiki/pages/{slug}")
}

func TestEndpointRegistersDeprecatedRepoWikiAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(wiki.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/wiki/pages")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/wiki/pages")
	assertRoute(t, server, http.MethodPatch, "/api/v1/repos/{id}/wiki/pages/{slug}")
	assertRoute(t, server, http.MethodDelete, "/api/v1/repos/{id}/wiki/pages/{slug}")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
