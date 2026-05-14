package lfs_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	lfs "github.com/lyonbrown4d/gity/internal/interfaces/http/lfs"
)

func TestEndpointRegistersProjectLFSManagementRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(lfs.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/lfs/objects")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/lfs/locks")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/lfs/locks")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/lfs/locks/{lock_id}/unlock")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
