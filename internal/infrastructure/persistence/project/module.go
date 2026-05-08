// Package project wires project persistence.
package project

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.project",
		dix.Description("Project persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectRepository),
		),
	)
}
