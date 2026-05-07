package job

import (
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/httpx"
)

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.job",
		dix.Description("Project job routes"),
		dix.Providers(
			dix.Provider5(NewEndpoint, dix.Into[httpx.Endpoint](dix.Order(110))),
		),
	)
}
