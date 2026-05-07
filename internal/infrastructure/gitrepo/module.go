package gitrepo

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.gitrepo",
		dix.Description("go-git repository read services"),
		dix.Providers(
			dix.Provider1(NewService),
		),
	)
}
