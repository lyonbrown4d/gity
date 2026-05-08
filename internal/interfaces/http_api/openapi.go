// Package httpapi contains shared HTTP API helpers.
package httpapi

import (
	"net/http"
	"strconv"
	"strings"

	"github.com/arcgolabs/httpx"
	"github.com/danielgtaylor/huma/v2"
)

const BearerAuthScheme = "BearerAuth"

func Configure(server httpx.ServerRuntime) {
	if server == nil {
		return
	}
	server.RegisterSecurityScheme(BearerAuthScheme, &huma.SecurityScheme{
		Type:         "http",
		Scheme:       "bearer",
		BearerFormat: "opaque",
	})
}

func EndpointSpec(prefix, tag, summaryPrefix, description string) httpx.EndpointSpec {
	return httpx.EndpointSpec{
		Prefix:        prefix,
		Tags:          httpx.Tags(tag),
		SummaryPrefix: summaryPrefix,
		Description:   description,
	}
}

func BearerSecurity() httpx.OpenAPISecurityRequirements {
	return httpx.SecurityRequirements(httpx.SecurityRequirement(BearerAuthScheme))
}

func Deprecated(reason string) httpx.OperationOption {
	return func(op *huma.Operation) {
		if op == nil {
			return
		}
		op.Deprecated = true
		reason = strings.TrimSpace(reason)
		if reason == "" {
			return
		}
		if strings.TrimSpace(op.Description) == "" {
			op.Description = reason
			return
		}
		if !strings.Contains(op.Description, reason) {
			op.Description += "\n\nDeprecated: " + reason
		}
	}
}

func markBearerProtected(op *huma.Operation) {
	if op == nil {
		return
	}
	if len(op.Security) == 0 {
		op.Security = []map[string][]string{{BearerAuthScheme: []string{}}}
	}
	ensureResponse(op, http.StatusUnauthorized)
	ensureResponse(op, http.StatusForbidden)
}

func ensureResponse(op *huma.Operation, status int) {
	if op.Responses == nil {
		op.Responses = map[string]*huma.Response{}
	}
	code := strconv.Itoa(status)
	if _, ok := op.Responses[code]; ok {
		return
	}
	op.Responses[code] = &huma.Response{Description: http.StatusText(status)}
}
