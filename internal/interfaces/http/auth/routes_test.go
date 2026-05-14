package auth_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	auth "github.com/lyonbrown4d/gity/internal/interfaces/http/auth"
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
