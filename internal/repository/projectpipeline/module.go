package projectpipeline

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpipeline",
		dix.Description("Project pipeline persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
