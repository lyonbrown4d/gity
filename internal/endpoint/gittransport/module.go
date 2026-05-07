package gittransport

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.gittransport",
		dix.Description("Raw Git Smart HTTP routes"),
		dix.Providers(
			dix.Provider2(NewRouteServices),
		),
		dix.Invokes(
			dix.Invoke6(RegisterRoutes),
		),
	)
}
