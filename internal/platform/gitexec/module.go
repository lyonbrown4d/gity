package gitexec

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"platform.gitexec",
		dix.Description("Native git command runner"),
		dix.Providers(
			dix.Provider1(NewRunner),
		),
	)
}
