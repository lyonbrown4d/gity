// Package projectissueassignee wires project issue assignee persistence.
package projectissueassignee

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissueassignee",
		dix.Description("Project issue assignee persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectIssueAssigneeRepository),
		),
	)
}
