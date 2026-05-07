package projectlfsobject

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectlfsobject",
		dix.Description("Project Git LFS object persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectLFSObjectRepository),
		),
	)
}
