// Package projectmergerequest wires project merge request persistence.
package projectmergerequest

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmergerequest",
		dix.Description("Project merge request persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMergeRequestRepository),
		),
	)
}
