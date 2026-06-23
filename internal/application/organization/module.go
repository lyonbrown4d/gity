// Package organization wires organization application services.
package organization

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.organization",
		dix.Description("Organization application services"),
		dix.Providers(
			dix.Provider4(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
