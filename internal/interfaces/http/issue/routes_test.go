package issue_test

import (
	"net/http"
	"testing"

	issue "github.com/DaiYuANg/gity/internal/interfaces/http/issue"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalIssueRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(issue.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/issues")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/issues")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/issues/{issue_iid}")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/issues/{issue_iid}/comments")
}

func TestEndpointRegistersDeprecatedRepoIssueAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(issue.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/issues")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/issues")
	assertRoute(t, server, http.MethodPatch, "/api/v1/repos/{id}/issues/{issue_iid}")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/issues/{issue_iid}/comments")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
