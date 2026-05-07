package projectjobartifact

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectjobartifact",
		dix.Description("Project job artifact persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectJobArtifactRepository),
		),
	)
}
