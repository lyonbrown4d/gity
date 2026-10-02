package todo_test

import (
	"net/http"
	"testing"

	"github.com/arcgolabs/httpx"
	todo "github.com/lyonbrown4d/gity/internal/interfaces/http/todo"
)

func TestRoutes(t *testing.T) {
	t.Parallel()

	server := httpx.New(httpx.WithBasePath("/api"))
	server.RegisterOnly(todo.NewEndpoint(nil, nil))

	assertRoute(t, server, http.MethodGet, "/api/v1/todos")
	assertRoute(t, server, http.MethodPost, "/api/v1/todos/{id}/done")
}

func assertRoute(t *testing.T, server httpx.ServerRuntime, method, path string) {
	t.Helper()
	if !server.HasRoute(method, path) {
		t.Fatalf("expected route %s %s", method, path)
	}
}
