package httpserver

import (
	"context"
	"log/slog"
	nethttp "net/http"
	"testing"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
)

type testEndpoint struct{}

type testEndpointOutput struct {
	Body struct {
		OK bool `json:"ok"`
	} `json:"body"`
}

func (e *testEndpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Test", "Test", "Test endpoint.")
}

func (e *testEndpoint) Register(registrar httpx.Registrar) {
	httpapi.MustRegisterRoutes(registrar, httpapi.Get("/probe", e.Probe))
}

func (e *testEndpoint) Probe(context.Context, *struct{}) (*testEndpointOutput, error) {
	out := &testEndpointOutput{}
	out.Body.OK = true
	return out, nil
}

func TestNewServerRegistersInjectedEndpoints(t *testing.T) {
	endpoints := collectionlist.NewList[httpx.Endpoint](&testEndpoint{})

	server, err := NewServer(NewFiberApp(), config.DefaultSettings(), slog.Default(), endpoints)
	if err != nil {
		t.Fatalf("new server: %v", err)
	}

	if !server.HasRoute(nethttp.MethodGet, "/api/v1/probe") {
		t.Fatalf("expected injected endpoint route")
	}
}
