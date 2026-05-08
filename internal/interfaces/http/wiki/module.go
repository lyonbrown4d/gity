// Package wiki wires wiki HTTP endpoints.
package wiki

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.wiki",
		dix.Description("Wiki routes"),
		dix.Providers(
			dix.Provider4(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(90))),
		),
	)
}
