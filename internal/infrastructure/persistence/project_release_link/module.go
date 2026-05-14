// Package projectreleaselink wires project release link persistence.
package projectreleaselink

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectreleaselink",
		dix.Description("Project release link persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectReleaseLinkRepository),
		),
	)
}
