// Package job wires project job application services.
package job

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.job",
		dix.Description("Project job application services"),
		dix.Providers(
			dix.Provider6(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
		),
	)
}
