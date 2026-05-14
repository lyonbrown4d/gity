// Package projectmergerequestapprovalrule wires merge request approval rule persistence.
package projectmergerequestapprovalrule

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmergerequestapprovalrule",
		dix.Description("Project merge request approval rule persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMergeRequestApprovalRuleRepository),
		),
	)
}
