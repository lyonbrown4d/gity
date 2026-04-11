package user

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.user",
		dix.Description("User routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
