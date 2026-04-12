package lfs

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.lfs",
		dix.Description("Git LFS application services"),
		dix.Providers(
			dix.Provider5(NewService),
		),
	)
}
