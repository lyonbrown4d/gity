package packageregistry

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.packageregistry",
		dix.Description("Package registry application services"),
		dix.Providers(
			dix.Provider5(NewService),
		),
	)
}
