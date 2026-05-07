package projectissue

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissue",
		dix.Description("Project issue persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
