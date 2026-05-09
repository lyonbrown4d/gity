// Package mergerequest wires merge request application services.
package mergerequest

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.mergerequest",
		dix.Description("Merge request application services"),
		dix.Providers(
			dix.Provider2(NewPipelineDeps),
			dix.Provider2(NewGitDependencies),
			dix.Provider5(NewRepositories),
			dix.Provider4(NewServiceWithDependencies),
		),
	)
}
