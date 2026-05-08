// Package wiki wires wiki application services.
package wiki

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.wiki",
		dix.Description("Project wiki application services"),
		dix.Providers(
			dix.Provider3(NewService),
		),
	)
}
