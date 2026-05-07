package mergerequest

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.mergerequest",
		dix.Description("Merge request application services"),
		dix.Providers(
			dix.Provider2(NewPipelineDeps),
			dix.Provider6(NewService),
		),
	)
}
