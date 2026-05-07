package user

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.user",
		dix.Description("User persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
