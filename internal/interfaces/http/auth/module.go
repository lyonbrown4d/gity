// Package auth wires auth HTTP endpoints.
package auth

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.auth",
		dix.Description("Authentication routes"),
		dix.Providers(
			dix.Provider1(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(20))),
		),
	)
}
