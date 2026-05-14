// Package release wires release HTTP endpoints.
package release

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.release",
		dix.Description("Release routes"),
		dix.Providers(
			dix.Provider3(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(95))),
		),
	)
}
