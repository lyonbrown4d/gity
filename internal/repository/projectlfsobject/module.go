package projectlfsobject

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectlfsobject",
		dix.Description("Project Git LFS object persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
