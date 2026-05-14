// Package projectaccesstoken wires project access token persistence.
package projectaccesstoken

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.project_access_token",
		dix.Description("Project access token persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectAccessTokenRepository),
		),
	)
}
