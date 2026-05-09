package audit_test

import (
	"net/http"
	"testing"

	audit "github.com/DaiYuANg/gity/internal/interfaces/http/audit"
	"github.com/arcgolabs/httpx"
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
