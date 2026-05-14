// Package pipeline wires CI pipeline application services.
package pipeline

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.pipeline",
		dix.Description("Project pipeline application services"),
		dix.Providers(
			dix.Provider3(NewRuntimeDeps),
			dix.Provider5(NewServiceFromDeps),
		),
	)
}
