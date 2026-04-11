package namespace

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.namespace",
		dix.Description("Namespace application services"),
		dix.Providers(
			dix.Provider2(NewService),
		),
	)
}
