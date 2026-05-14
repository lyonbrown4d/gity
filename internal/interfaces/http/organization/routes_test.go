package organization_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	organization "github.com/lyonbrown4d/gity/internal/interfaces/http/organization"
)

func TestEndpointRegistersOrganizationRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(organization.NewEndpoint(nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/orgs")
	assertRoute(t, server, http.MethodPost, "/api/v1/orgs")
	assertRoute(t, server, http.MethodGet, "/api/v1/orgs/{id}/members")
	assertRoute(t, server, http.MethodPost, "/api/v1/orgs/{id}/members")
	assertNoRoute(t, server, http.MethodGet, "/api/v1/organizations")
	assertNoRoute(t, server, http.MethodPost, "/api/v1/organizations")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}

func assertNoRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if server.HasRoute(method, path) {
		t.Fatalf("unexpected route %s %s", method, path)
	}
}
