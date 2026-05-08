// Package system wires system HTTP endpoints.
package system

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.system",
		dix.Description("System routes"),
		dix.Providers(
			dix.Provider1(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(10))),
		),
	)
}
