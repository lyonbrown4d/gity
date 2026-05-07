package project

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.project",
		dix.Description("Project application services"),
		dix.Providers(
			dix.Provider6(NewService),
		),
	)
}
