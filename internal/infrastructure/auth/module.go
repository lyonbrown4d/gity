package auth

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.auth",
		dix.Description("Authentication runtime"),
		dix.Providers(
			dix.Provider3(NewTokenProvider),
			dix.Provider2(NewProjectAuthorizer),
			dix.Provider5(NewEngine),
			dix.Provider1(NewRuntime),
		),
	)
}
