package usertoken

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.usertoken",
		dix.Description("User token persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewUserAccessTokenRepository),
		),
	)
}
