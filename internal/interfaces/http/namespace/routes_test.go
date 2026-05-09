package namespace_test

import (
	"net/http"
	"testing"

	namespace "github.com/DaiYuANg/gity/internal/interfaces/http/namespace"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersNamespaceRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(namespace.NewEndpoint(nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/namespaces")
	assertRoute(t, server, http.MethodPost, "/api/v1/namespaces")
	assertRoute(t, server, http.MethodGet, "/api/v1/namespaces/{id}/members")
	assertRoute(t, server, http.MethodPost, "/api/v1/namespaces/{id}/members")
}

func TestEndpointRegistersOrganizationAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(namespace.NewEndpoint(nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/orgs")
	assertRoute(t, server, http.MethodPost, "/api/v1/orgs")
	assertRoute(t, server, http.MethodGet, "/api/v1/orgs/{id}/members")
	assertRoute(t, server, http.MethodPost, "/api/v1/orgs/{id}/members")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
