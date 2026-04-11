package system

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.system",
		dix.Description("System routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
