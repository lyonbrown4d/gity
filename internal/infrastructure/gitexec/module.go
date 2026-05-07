package gitexec

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.gitexec",
		dix.Description("Native git command runner"),
		dix.Providers(
			dix.Provider1(NewRunner),
		),
	)
}
