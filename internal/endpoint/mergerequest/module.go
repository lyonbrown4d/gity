package mergerequest

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.mergerequest",
		dix.Description("Merge request routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
