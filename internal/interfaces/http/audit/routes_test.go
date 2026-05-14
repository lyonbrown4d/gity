package audit_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	audit "github.com/lyonbrown4d/gity/internal/interfaces/http/audit"
)

func TestRoutes(t *testing.T) {
	t.Parallel()

	server := httpx.New(httpx.WithBasePath("/api"))
	server.RegisterOnly(audit.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/audit-events")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
