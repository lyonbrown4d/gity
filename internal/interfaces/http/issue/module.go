// Package issue wires issue HTTP endpoints.
package issue

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.issue",
		dix.Description("Issue routes"),
		dix.Providers(
			dix.Provider4(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(60))),
		),
		dix.Invokes(
			dix.Invoke3(RegisterMultipartRoutes),
		),
	)
}
