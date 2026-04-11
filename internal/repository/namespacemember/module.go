package namespacemember

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.namespacemember",
		dix.Description("Namespace member persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
		),
	)
}
