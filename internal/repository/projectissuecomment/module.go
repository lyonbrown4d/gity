package projectissuecomment

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissuecomment",
		dix.Description("Project issue comment persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
