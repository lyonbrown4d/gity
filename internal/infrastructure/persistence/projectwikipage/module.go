package projectwikipage

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectwikipage",
		dix.Description("Project wiki page persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
