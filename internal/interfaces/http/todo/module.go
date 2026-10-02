// Package todo wires todo HTTP endpoints.
package todo

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.todo",
		dix.Description("Todo routes"),
		dix.Providers(
			dix.Provider2(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(82))),
		),
	)
}
