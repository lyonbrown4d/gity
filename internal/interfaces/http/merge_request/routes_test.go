package mergerequest_test

import (
	"net/http"
	"testing"

	mergerequest "github.com/DaiYuANg/gity/internal/interfaces/http/merge_request"
	"github.com/arcgolabs/httpx"
)

func TestEndpointRegistersCanonicalMergeRequestRoutes(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(mergerequest.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/merge-requests")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/merge-requests")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/merge-requests/{merge_iid}/diff")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/merge-requests/{merge_iid}/checks")
	assertRoute(t, server, http.MethodGet, "/api/v1/projects/{id}/merge-requests/{merge_iid}/participants")
	assertRoute(t, server, http.MethodPost, "/api/v1/projects/{id}/merge-requests/{merge_iid}/merge")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/merge-requests/{merge_iid}/reviewers")
	assertRoute(t, server, http.MethodPatch, "/api/v1/projects/{id}/merge-requests/{merge_iid}/assignees")
}

func TestEndpointRegistersDeprecatedRepoMergeRequestAliases(t *testing.T) {
	server := httpx.New(httpx.WithBasePath("/api"))

	server.RegisterOnly(mergerequest.NewEndpoint(nil, nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/merge-requests")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/merge-requests")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/merge-requests/{merge_iid}/diff")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/merge-requests/{merge_iid}/checks")
	assertRoute(t, server, http.MethodGet, "/api/v1/repos/{id}/merge-requests/{merge_iid}/participants")
	assertRoute(t, server, http.MethodPost, "/api/v1/repos/{id}/merge-requests/{merge_iid}/merge")
	assertRoute(t, server, http.MethodPatch, "/api/v1/repos/{id}/merge-requests/{merge_iid}/reviewers")
	assertRoute(t, server, http.MethodPatch, "/api/v1/repos/{id}/merge-requests/{merge_iid}/assignees")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
