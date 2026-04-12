package projectissueattachment

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissueattachment",
		dix.Description("Project issue attachment persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
