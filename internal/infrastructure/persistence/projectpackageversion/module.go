package projectpackageversion

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpackageversion",
		dix.Description("Project package version persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
