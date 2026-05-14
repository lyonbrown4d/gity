package system_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	"github.com/lyonbrown4d/gity/internal/config"
	system "github.com/lyonbrown4d/gity/internal/interfaces/http/system"
)

func TestEndpointRegistersSystemRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(system.NewEndpoint(config.Settings{}))

	assertRoute(t, server, http.MethodGet, "/api/health")
	assertRoute(t, server, http.MethodGet, "/api/v1/rewrite/info")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
