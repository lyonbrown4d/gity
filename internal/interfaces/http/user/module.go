// Package user wires user HTTP endpoints.
package user

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.user",
		dix.Description("User routes"),
		dix.Providers(
			dix.Provider2(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(30))),
		),
	)
}
