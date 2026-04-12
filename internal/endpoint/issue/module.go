package issue

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.issue",
		dix.Description("Issue routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
