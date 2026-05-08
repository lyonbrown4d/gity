// Package projectpipelinejob wires project pipeline job persistence.
package projectpipelinejob

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpipelinejob",
		dix.Description("Project pipeline job persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectPipelineJobRepository),
		),
	)
}
