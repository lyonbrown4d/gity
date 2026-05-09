// Package mergerequest wires merge request HTTP endpoints.
package mergerequest

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.mergerequest",
		dix.Description("Merge request routes"),
		dix.Providers(
			dix.Provider4(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(70))),
		),
	)
}
