package runner

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.runner",
		dix.Description("Project runner routes"),
		dix.Providers(
			dix.Provider3(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(120))),
		),
	)
}
