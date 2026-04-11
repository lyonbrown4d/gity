package project

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.project",
		dix.Description("Project routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
