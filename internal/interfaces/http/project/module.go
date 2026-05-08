// Package project wires project HTTP endpoints.
package project

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.project",
		dix.Description("Project routes"),
		dix.Providers(
			dix.Provider4(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(50))),
		),
	)
}
