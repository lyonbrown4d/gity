package namespace

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.namespace",
		dix.Description("Namespace application services"),
		dix.Providers(
			dix.Provider4(NewService),
		),
	)
}
