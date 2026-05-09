// Package projectissuelabel wires project issue label persistence.
package projectissuelabel

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissuelabel",
		dix.Description("Project issue label persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectIssueLabelRepository),
		),
	)
}
