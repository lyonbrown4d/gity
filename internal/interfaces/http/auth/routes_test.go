package auth_test

import (
	"net/http"
	"testing"

	auth "github.com/DaiYuANg/gity/internal/interfaces/http/auth"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersAuthRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(auth.NewEndpoint(nil))

	assertRoute(t, server, http.MethodPost, "/api/v1/auth/login")
	assertRoute(t, server, http.MethodPost, "/api/v1/auth/refresh")
	assertRoute(t, server, http.MethodPost, "/api/v1/auth/logout")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
