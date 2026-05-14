// Package projectdeploykey wires project deploy key persistence.
package projectdeploykey

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.project_deploy_key",
		dix.Description("Project deploy key persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectDeployKeyRepository),
		),
	)
}
