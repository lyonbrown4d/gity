package pipeline

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.pipeline",
		dix.Description("Project pipeline application services"),
		dix.Providers(
			dix.Provider5(NewService),
		),
	)
}
