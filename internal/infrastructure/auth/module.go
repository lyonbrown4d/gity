package auth

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.auth",
		dix.Description("Authentication runtime"),
		dix.Providers(
			dix.Provider1(NewEngine),
			dix.Provider1(NewRuntime),
		),
	)
}
