// Package projectmember wires project member persistence.
package projectmember

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmember",
		dix.Description("Project member persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMemberRepository),
		),
	)
}
