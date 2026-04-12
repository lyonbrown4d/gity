package gittransport

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.gittransport",
		dix.Description("Raw Git Smart HTTP routes"),
		dix.Invokes(
			dix.Invoke5(RegisterRoutes),
		),
	)
}
