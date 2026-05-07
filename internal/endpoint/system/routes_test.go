package system

import (
	"net/http"
	"testing"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersSystemRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(NewEndpoint(config.Settings{}))

	assertRoute(t, server, http.MethodGet, "/api/health")
	assertRoute(t, server, http.MethodGet, "/api/v1/rewrite/info")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method string, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
