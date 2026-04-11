package project

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.project",
		dix.Description("Project application services"),
		dix.Providers(
			dix.Provider4(NewService),
		),
	)
}
