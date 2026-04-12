package mergerequest

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.mergerequest",
		dix.Description("Merge request application services"),
		dix.Providers(
			dix.Provider4(NewService),
		),
	)
}
