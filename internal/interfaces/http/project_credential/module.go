// Package projectcredential exposes project credential HTTP APIs.
package projectcredential

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.project_credential",
		dix.Description("Project credential HTTP endpoints"),
		dix.Providers(
			dix.Provider3(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(96))),
		),
	)
}
