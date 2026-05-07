package project

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.project",
		dix.Description("Project persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
