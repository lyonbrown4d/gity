// Package audit wires audit HTTP endpoints.
package audit

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.audit",
		dix.Description("Audit routes"),
		dix.Providers(
			dix.Provider3(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(75))),
		),
	)
}
