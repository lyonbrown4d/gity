// Package issue wires issue application services.
package issue

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.issue",
		dix.Description("Issue application services"),
		dix.Providers(
			dix.Provider6(NewRepositories),
			dix.Provider3(NewRuntimeDependencies),
			dix.Provider2(NewServiceWithDependencies),
		),
	)
}
