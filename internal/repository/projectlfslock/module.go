package projectlfslock

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectlfslock",
		dix.Description("Project Git LFS lock persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
