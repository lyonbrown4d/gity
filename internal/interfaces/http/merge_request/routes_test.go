package mergerequest_test

import (
	"fmt"
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	mergerequest "github.com/lyonbrown4d/gity/internal/interfaces/http/merge_request"
)

func TestEndpointRegistersMergeRequestRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(mergerequest.NewEndpoint(nil, nil, nil, nil))

	assertMergeRequestRoutes(t, server, "projects")
	assertMergeRequestRoutes(t, server, "repos")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/merge-request-approval-rules")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/merge-request-approval-rules")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/merge-request-approval-rules/{rule_id}")
	assertRoute(t, server, http.MethodDelete, "/api/v1/projects/{id}/merge-request-approval-rules/{rule_id}")
}

func assertMergeRequestRoutes(t *testing.T, server httpx.ServerRuntime, scope string) {
	t.Helper()

	routes := []struct {
		method string
		path   string
	}{
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests"},
		{http.MethodPost, "/api/v1/%s/{id}/merge-requests"},
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests/{merge_iid}/diff"},
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests/{merge_iid}/checks"},
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests/{merge_iid}/participants"},
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests/{merge_iid}/comments"},
		{http.MethodPost, "/api/v1/%s/{id}/merge-requests/{merge_iid}/comments"},
		{http.MethodGet, "/api/v1/%s/{id}/merge-requests/{merge_iid}/approvals"},
		{http.MethodPost, "/api/v1/%s/{id}/merge-requests/{merge_iid}/approve"},
		{http.MethodPost, "/api/v1/%s/{id}/merge-requests/{merge_iid}/unapprove"},
		{http.MethodPost, "/api/v1/%s/{id}/merge-requests/{merge_iid}/merge"},
		{http.MethodPatch, "/api/v1/%s/{id}/merge-requests/{merge_iid}/reviewers"},
		{http.MethodPatch, "/api/v1/%s/{id}/merge-requests/{merge_iid}/assignees"},
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
