// Package projectcivariable wires project CI variable persistence.
package projectcivariable

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectcivariable",
		dix.Description("Project CI variable persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectCIVariableRepository),
		),
	)
}
