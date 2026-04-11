package namespace

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.namespace",
		dix.Description("Namespace routes"),
		dix.Invokes(
			dix.Invoke2(RegisterRoutes),
		),
	)
}
