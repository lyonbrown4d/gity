// Package projectrelease wires project release persistence.
package projectrelease

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectrelease",
		dix.Description("Project release persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectReleaseRepository),
		),
	)
}
