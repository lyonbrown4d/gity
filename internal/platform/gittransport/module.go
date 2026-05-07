package gittransport

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"platform.gittransport",
		dix.Description("Git transport services"),
		dix.Providers(
			dix.Provider1(NewService),
		),
	)
}
