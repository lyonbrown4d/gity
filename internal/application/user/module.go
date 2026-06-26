// Package user wires user application services.
package user

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.user",
		dix.Description("User application services"),
		dix.Providers(
			dix.Provider5(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
