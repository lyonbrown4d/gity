package auth

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"platform.auth",
		dix.Description("Authentication runtime"),
		dix.Providers(
			dix.Provider3(NewRuntime),
		),
	)
}
