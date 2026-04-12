package projectpackage

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpackage",
		dix.Description("Project package persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
