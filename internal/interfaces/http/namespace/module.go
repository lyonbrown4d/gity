// Package namespace wires namespace HTTP endpoints.
package namespace

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.namespace",
		dix.Description("Namespace routes"),
		dix.Providers(
			dix.Provider1(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(40))),
		),
	)
}
