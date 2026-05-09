package httpapi_test

import (
	"testing"

	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

func TestConfigureRegistersBearerAuthScheme(t *testing.T) {
	server := httpx.New()

	httpapi.Configure(server)

	doc := server.OpenAPI()
	if doc == nil || doc.Components == nil || doc.Components.SecuritySchemes == nil {
		t.Fatalf("expected OpenAPI security components")
	}
	scheme, ok := doc.Components.SecuritySchemes[httpapi.BearerAuthScheme]
	if !ok {
		t.Fatalf("expected %s security scheme", httpapi.BearerAuthScheme)
	}
	if scheme.Type != "http" || scheme.Scheme != "bearer" {
		t.Fatalf("unexpected security scheme: %#v", scheme)
	}
}
