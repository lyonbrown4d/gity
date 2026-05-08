// Package projectjob wires project job persistence.
package projectjob

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectjob",
		dix.Description("Project job persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectJobRepository),
		),
	)
}
