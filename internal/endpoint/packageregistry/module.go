package packageregistry

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.packageregistry",
		dix.Description("Package registry routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
