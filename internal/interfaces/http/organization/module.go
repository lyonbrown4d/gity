// Package organization wires organization HTTP endpoints.
package organization

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.organization",
		dix.Description("Organization routes"),
		dix.Providers(
			dix.Provider2(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(40))),
		),
	)
}
