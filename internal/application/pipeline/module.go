// Package pipeline wires CI pipeline application services.
package pipeline

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.pipeline",
		dix.Description("Project pipeline application services"),
		dix.Providers(
			dix.Provider4(NewDependencies),
			dix.Provider3(NewRuntimeDependencies),
			dix.Provider2(NewServiceWithDependencies),
		),
	)
}
