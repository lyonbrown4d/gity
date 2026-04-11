package project

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.project",
		dix.Description("Project persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
