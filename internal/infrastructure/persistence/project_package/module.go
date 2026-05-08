// Package projectpackage wires project package persistence.
package projectpackage

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectpackage",
		dix.Description("Project package persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectPackageRepository),
		),
	)
}
