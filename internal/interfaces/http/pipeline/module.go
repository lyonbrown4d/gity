// Package pipeline wires CI pipeline HTTP endpoints.
package pipeline

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.pipeline",
		dix.Description("Project pipeline routes"),
		dix.Providers(
			dix.Provider4(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(100))),
		),
	)
}
