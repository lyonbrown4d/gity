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
	assertRoute(t, server, http.MethodPut, "/api/v1/projects/{id}/packages/generic/{package_name}/{package_version}/{file_name...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/generic/{package_name}/{package_version}/{file_name...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/nuget/index.json")
	assertRoute(t, server, http.MethodPut, "/api/v1/projects/{id}/packages/nuget/{package_name}/{package_version}/{file_name...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/nuget/{package_name}/{package_version}/{file_name...}")
	assertRoute(t, server, http.MethodPut, "/api/v1/projects/{id}/packages/maven/{file_path...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/maven/{file_path...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/npm/{package_name...}")
	assertRoute(t, server, http.MethodPut, "/api/v1/projects/{id}/packages/npm/{package_name...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/pypi/simple")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/pypi/simple/{package_name}")
	assertRoute(t, server, http.MethodPut, "/api/v1/projects/{id}/packages/pypi/{package_name}/{package_version}/{file_name...}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/{package_id}")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/packages/files")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/files/{file_id}")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/packages/files/{file_id}/download")
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
