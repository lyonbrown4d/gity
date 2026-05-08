// Package projectbranchprotection wires project branch protection persistence.
package projectbranchprotection

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectbranchprotection",
		dix.Description("Project branch protection persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectBranchProtectionRepository),
		),
	)
}
