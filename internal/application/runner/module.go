// Package runner wires runner application services.
package runner

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.runner",
		dix.Description("Project runner application services"),
		dix.Providers(
			dix.Provider6(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
