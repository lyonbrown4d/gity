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

	assertDownloadResponses(t, server, "/api/v1/projects/{id}/packages/generic/{package_name}/{package_version}/{file_name}")
	assertDownloadResponses(t, server, "/api/v1/projects/{id}/packages/nuget/{package_name}/{package_version}/{file_name}")
	assertDownloadResponses(t, server, "/api/v1/projects/{id}/packages/maven/{file_path}")
	assertDownloadResponses(t, server, "/api/v1/projects/{id}/packages/files/{file_id}/download")
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

func assertDownloadResponses(t *testing.T, server httpx.ServerRuntime, path string) {
	t.Helper()
	operation := server.OpenAPI().Paths[path].Get
	if operation == nil {
		t.Fatalf("expected GET operation for %s", path)
	}
	for _, status := range []string{"200", "206", "304", "416"} {
		if _, ok := operation.Responses[status]; !ok {
			t.Errorf("GET %s does not document response %s", path, status)
		}
	}
}
