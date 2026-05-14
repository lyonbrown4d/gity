package packageregistry_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	packageregistry "github.com/lyonbrown4d/gity/internal/interfaces/http/package_registry"
)

func TestEndpointRegistersCanonicalPackageRegistryRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(packageregistry.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/{package_id}")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/packages/files")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/files/{file_id}")
}

func TestEndpointRegistersDeprecatedRepoPackageRegistryAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(packageregistry.NewEndpoint(nil, nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/packages")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/packages/{package_id}")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/packages/files")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/packages/files/{file_id}")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
