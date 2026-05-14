// Package release wires release application services.
package release

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.release",
		dix.Description("Project release application services"),
		dix.Providers(
			dix.Provider6(NewService),
		),
	)
}
