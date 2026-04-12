package projectpackageversion

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpackageversion",
		dix.Description("Project package version persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
