// Package projectmergerequestapproval wires project merge request approval persistence.
package projectmergerequestapproval

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmergerequestapproval",
		dix.Description("Project merge request approval persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMergeRequestApprovalRepository),
		),
	)
}
