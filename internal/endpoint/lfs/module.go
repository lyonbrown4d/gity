package lfs

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.lfs",
		dix.Description("Git LFS routes"),
		dix.Invokes(
			dix.Invoke5(RegisterRoutes),
		),
	)
}
