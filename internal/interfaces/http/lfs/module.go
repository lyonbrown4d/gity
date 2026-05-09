// Package lfs wires Git LFS HTTP endpoints.
package lfs

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.lfs",
		dix.Description("Git LFS routes"),
		dix.Providers(
			dix.Provider3(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(85))),
		),
		dix.Invokes(
			dix.Invoke5(RegisterRoutes),
		),
	)
}
