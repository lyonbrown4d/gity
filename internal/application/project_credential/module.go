// Package projectcredential implements project access token, deploy token and deploy key use cases.
package projectcredential

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.project_credential",
		dix.Description("Project credential application services"),
		dix.Providers(
			dix.Provider3(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
