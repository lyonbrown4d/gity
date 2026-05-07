package runner

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.runner",
		dix.Description("Project runner application services"),
		dix.Providers(
			dix.Provider4(NewService),
		),
	)
}
