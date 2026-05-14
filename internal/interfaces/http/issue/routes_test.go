package issue_test

import (
	"fmt"
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	issue "github.com/lyonbrown4d/gity/internal/interfaces/http/issue"
)

func TestEndpointRegistersIssueRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(issue.NewEndpoint(nil, nil, nil, nil))

	assertIssueRoutes(t, server, "projects")
	assertIssueRoutes(t, server, "repos")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/issues/by-number/{issue_iid}")
}

func assertIssueRoutes(t *testing.T, server httpx.ServerRuntime, scope string) {
	t.Helper()

	routes := []struct {
		method string
		path   string
	}{
		{http.MethodGet, "/api/v1/%s/{id}/issues"},
		{http.MethodPost, "/api/v1/%s/{id}/issues"},
		{http.MethodPatch, "/api/v1/%s/{id}/issues/{issue_iid}"},
		{http.MethodGet, "/api/v1/%s/{id}/issues/{issue_iid}/comments"},
		{http.MethodPost, "/api/v1/%s/{id}/issues/{issue_iid}/comments"},
		{http.MethodGet, "/api/v1/%s/{id}/issues/{issue_iid}/assignees"},
		{http.MethodPatch, "/api/v1/%s/{id}/issues/{issue_iid}/assignees"},
		{http.MethodGet, "/api/v1/%s/{id}/issues/{issue_iid}/labels"},
		{http.MethodPatch, "/api/v1/%s/{id}/issues/{issue_iid}/labels"},
		{http.MethodGet, "/api/v1/%s/{id}/issues/{issue_iid}/attachments"},
		{http.MethodPost, "/api/v1/%s/{id}/issues/{issue_iid}/attachments"},
		{http.MethodGet, "/api/v1/%s/{id}/issues/{issue_iid}/attachments/{attachment_id}"},
	}
	for _, route := range routes {
		assertRoute(t, server, route.method, fmt.Sprintf(route.path, scope))
	}
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
