package projectpackagefile

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpackagefile",
		dix.Description("Project package file persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
