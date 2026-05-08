// Package project wires project application services.
package project

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.project",
		dix.Description("Project application services"),
		dix.Providers(
			dix.Provider2(NewGitDependencies),
			dix.Provider6(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
