package httpapi

import (
	"testing"

	"github.com/arcgolabs/httpx"
)

func TestConfigureRegistersBearerAuthScheme(t *testing.T) {
	server := httpx.New()

	Configure(server)

	doc := server.OpenAPI()
	if doc == nil || doc.Components == nil || doc.Components.SecuritySchemes == nil {
		t.Fatalf("expected OpenAPI security components")
	}
	scheme, ok := doc.Components.SecuritySchemes[BearerAuthScheme]
	if !ok {
		t.Fatalf("expected %s security scheme", BearerAuthScheme)
	}
	if scheme.Type != "http" || scheme.Scheme != "bearer" {
		t.Fatalf("unexpected security scheme: %#v", scheme)
	}
}
