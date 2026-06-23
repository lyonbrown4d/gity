// Package packageregistry wires package registry application services.
package packageregistry

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.packageregistry",
		dix.Description("Package registry application services"),
		dix.Providers(
			dix.Provider5(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
