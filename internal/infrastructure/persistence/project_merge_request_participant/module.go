// Package projectmergerequestparticipant wires merge request participant persistence.
package projectmergerequestparticipant

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmergerequestparticipant",
		dix.Description("Project merge request participant persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMergeRequestParticipantRepository),
		),
	)
}
