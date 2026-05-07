package projectrunner

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectrunner",
		dix.Description("Project runner persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
